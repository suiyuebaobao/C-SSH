//! 加载本机证书，固定 TLS 1.3，拒绝明文代理入口。

use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use anyhow::{Context, Result, bail};
use rustls::{ServerConfig, pki_types::PrivateKeyDer};
use tokio_rustls::TlsAcceptor;

pub(crate) fn acceptor(cert_file: &Path, key_file: &Path) -> Result<TlsAcceptor> {
    let mut cert_reader =
        BufReader::new(File::open(cert_file).with_context(|| "无法打开网关 TLS 证书文件")?);
    let certificates = rustls_pemfile::certs(&mut cert_reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("无法解析网关 TLS 证书")?;
    if certificates.is_empty() {
        bail!("网关 TLS 证书链为空");
    }

    let mut key_reader =
        BufReader::new(File::open(key_file).with_context(|| "无法打开网关 TLS 私钥文件")?);
    let private_key: PrivateKeyDer<'static> = rustls_pemfile::private_key(&mut key_reader)
        .context("无法解析网关 TLS 私钥")?
        .context("网关 TLS 私钥文件中没有受支持的私钥")?;

    let mut config = ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_no_client_auth()
        .with_single_cert(certificates, private_key)
        .context("网关 TLS 证书与私钥不匹配")?;
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(TlsAcceptor::from(Arc::new(config)))
}
