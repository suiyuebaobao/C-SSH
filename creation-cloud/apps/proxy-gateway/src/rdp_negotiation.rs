//! 有界验证 RDP 的 TPKT／X.224 协商，原样交付后才允许后续端到端字节流。

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zeroize::Zeroizing;

// X.224 LI=255 保留，CR／CC 没有额外 user data，因此整帧最多 4+1+254 字节。
const MAX_PACKET_BYTES: usize = 259;
const X224_HEADER_BYTES: usize = 11;
const SECURITY_PROTOCOL_MASK: u32 = 0x1f;

#[derive(Debug, Error)]
pub(crate) enum NegotiationError {
    #[error("RDP negotiation invalid")]
    Invalid,
    #[error("RDP negotiation unavailable")]
    Unavailable,
    #[error("RDP negotiation refused by server")]
    Refused,
}

pub(crate) async fn negotiate<C, T>(
    client: &mut C,
    target: &mut T,
    prefetched: &[u8],
) -> Result<(), NegotiationError>
where
    C: AsyncRead + AsyncWrite + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut pending = prefetched;
    let request = read_packet(&mut (&mut pending).chain(&mut *client)).await?;
    let protocols = validate_request(&request)?;
    write_packet(target, &request).await?;
    let confirm = read_packet(target).await?;
    let accepted = validate_confirm(&confirm, protocols)?;
    write_packet(client, &confirm).await?;
    if !accepted {
        // 合法失败帧交给端到端 RDP 引擎解释，但不能把失败协商升级成通用 TCP。
        return Err(NegotiationError::Refused);
    }
    // HTTP 首次读取可能已带后续数据；只有双向协商都通过后才交给目标。
    if !pending.is_empty() {
        write_packet(target, pending).await?;
    }
    Ok(())
}

async fn write_packet<W>(writer: &mut W, packet: &[u8]) -> Result<(), NegotiationError>
where
    W: AsyncWrite + Unpin,
{
    writer
        .write_all(packet)
        .await
        .map_err(|_| NegotiationError::Unavailable)?;
    writer
        .flush()
        .await
        .map_err(|_| NegotiationError::Unavailable)
}

async fn read_packet<R>(reader: &mut R) -> Result<Zeroizing<Vec<u8>>, NegotiationError>
where
    R: AsyncRead + Unpin,
{
    let mut header = [0_u8; 4];
    reader
        .read_exact(&mut header)
        .await
        .map_err(|_| NegotiationError::Unavailable)?;
    let length = usize::from(u16::from_be_bytes([header[2], header[3]]));
    if header[..2] != [3, 0] || !(X224_HEADER_BYTES..=MAX_PACKET_BYTES).contains(&length) {
        return Err(NegotiationError::Invalid);
    }
    let mut packet = Zeroizing::new(vec![0_u8; length]);
    packet[..4].copy_from_slice(&header);
    reader
        .read_exact(&mut packet[4..])
        .await
        .map_err(|_| NegotiationError::Unavailable)?;
    Ok(packet)
}

fn x224_body(packet: &[u8], kind: u8) -> Result<&[u8], NegotiationError> {
    if packet.len() < X224_HEADER_BYTES
        || packet.len() > MAX_PACKET_BYTES
        || packet[..2] != [3, 0]
        || usize::from(u16::from_be_bytes([packet[2], packet[3]])) != packet.len()
        || packet[4] == 255
        || usize::from(packet[4]) + 5 != packet.len()
        || packet[5] != kind
        || packet[10] != 0
    {
        return Err(NegotiationError::Invalid);
    }
    // RDP 使用 Class 0；DST-REF/SRC-REF 不参与 RDP，服务端可返回非零引用。
    Ok(&packet[X224_HEADER_BYTES..])
}

fn validate_request(packet: &[u8]) -> Result<u32, NegotiationError> {
    let mut body = x224_body(packet, 0xe0)?;
    if body.starts_with(b"Cookie: mstshash=") || body.starts_with(b"Cookie: msts=") {
        let line_end = body
            .windows(2)
            .position(|pair| pair == b"\r\n")
            .ok_or(NegotiationError::Invalid)?;
        if !body[..line_end]
            .iter()
            .all(|byte| (b' '..=b'~').contains(byte))
        {
            return Err(NegotiationError::Invalid);
        }
        body = &body[line_end + 2..];
    }
    // 本产品总会发送 RDP_NEG_REQ；缺失协商的普通 X.224 流不能作为 RDP admission。
    if body.len() < 8 || body[0] != 1 || body[1] & !0x0b != 0 || body[2..4] != [8, 0] {
        return Err(NegotiationError::Invalid);
    }
    let protocols = u32::from_le_bytes(
        body[4..8]
            .try_into()
            .map_err(|_| NegotiationError::Invalid)?,
    );
    if protocols & !SECURITY_PROTOCOL_MASK != 0 {
        return Err(NegotiationError::Invalid);
    }
    let correlation = body[1] & 0x08 != 0;
    if correlation {
        if body.len() != 44 || body[8..12] != [6, 0, 36, 0] || body[28..44] != [0; 16] {
            return Err(NegotiationError::Invalid);
        }
    } else if body.len() != 8 {
        return Err(NegotiationError::Invalid);
    }
    Ok(protocols)
}

fn validate_confirm(packet: &[u8], requested: u32) -> Result<bool, NegotiationError> {
    let body = x224_body(packet, 0xd0)?;
    if body.len() != 8 || body[2..4] != [8, 0] {
        return Err(NegotiationError::Invalid);
    }
    let selected = u32::from_le_bytes(
        body[4..8]
            .try_into()
            .map_err(|_| NegotiationError::Invalid)?,
    );
    match body[0] {
        2 if selected == 0 || (selected.is_power_of_two() && selected & requested == selected) => {
            Ok(true)
        }
        3 if body[1] == 0 && (1..=6).contains(&selected) => Ok(false),
        _ => Err(NegotiationError::Invalid),
    }
}

#[cfg(test)]
#[path = "rdp_negotiation_tests.rs"]
mod tests;
