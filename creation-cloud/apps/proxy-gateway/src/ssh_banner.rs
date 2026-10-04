//! 在开放隧道前有界读取并验证目标端 SSH identification，拒绝通用 TCP 服务。

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

const MAX_BANNER_BYTES: usize = 4 * 1024;
const MAX_LINE_BYTES: usize = 255;

#[derive(Debug, Error)]
pub(crate) enum BannerError {
    #[error("SSH banner invalid")]
    Invalid,
    #[error("SSH banner unavailable")]
    Unavailable,
}

pub(crate) async fn read<R>(reader: &mut R) -> Result<Vec<u8>, BannerError>
where
    R: AsyncRead + Unpin,
{
    let mut buffer = Vec::with_capacity(MAX_LINE_BYTES);
    let mut line_start = 0;
    loop {
        if buffer.len() >= MAX_BANNER_BYTES {
            return Err(BannerError::Invalid);
        }
        let mut byte = [0_u8; 1];
        let count = reader
            .read(&mut byte)
            .await
            .map_err(|_| BannerError::Unavailable)?;
        if count == 0 {
            return Err(BannerError::Invalid);
        }
        buffer.push(byte[0]);
        let line_length = buffer.len() - line_start;
        if line_length > MAX_LINE_BYTES {
            return Err(BannerError::Invalid);
        }
        if byte[0] != b'\n' {
            continue;
        }

        let line = &buffer[line_start..];
        if line.starts_with(b"SSH-") {
            return validate_identification(line)
                .then_some(buffer)
                .ok_or(BannerError::Invalid);
        }
        line_start = buffer.len();
    }
}

fn validate_identification(line: &[u8]) -> bool {
    if !line.ends_with(b"\r\n") {
        return false;
    }
    let body = &line[..line.len() - 2];
    let prefix = if body.starts_with(b"SSH-2.0-") {
        b"SSH-2.0-".as_slice()
    } else if body.starts_with(b"SSH-1.99-") {
        b"SSH-1.99-".as_slice()
    } else {
        return false;
    };
    let software_end = body
        .iter()
        .position(|byte| *byte == b' ')
        .unwrap_or(body.len());
    software_end > prefix.len() && body.iter().all(|byte| (b' '..=b'~').contains(byte))
}
