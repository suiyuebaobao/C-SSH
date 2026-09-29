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

#[cfg(test)]
mod tests {
    use tokio::io::{AsyncWriteExt, duplex};

    use super::{BannerError, read};

    #[tokio::test]
    async fn accepts_bounded_ssh2_banner_with_preface() {
        let (mut writer, mut reader) = duplex(512);
        writer
            .write_all(b"authorized access only\r\nSSH-2.0-OpenSSH_9.9 test\r\n")
            .await
            .expect("写入应成功");
        drop(writer);
        let banner = read(&mut reader).await.expect("SSH banner 应有效");
        assert_eq!(
            banner,
            b"authorized access only\r\nSSH-2.0-OpenSSH_9.9 test\r\n"
        );
    }

    #[tokio::test]
    async fn rejects_non_ssh_and_oversized_banner() {
        for raw in [
            b"HTTP/1.1 200 OK\r\n\r\n".to_vec(),
            [vec![b'x'; 256], b"\r\nSSH-2.0-test\r\n".to_vec()].concat(),
            b"SSH-1.5-obsolete\r\n".to_vec(),
        ] {
            let (mut writer, mut reader) = duplex(raw.len() + 1);
            writer.write_all(&raw).await.expect("写入应成功");
            drop(writer);
            assert!(matches!(read(&mut reader).await, Err(BannerError::Invalid)));
        }
    }
}
