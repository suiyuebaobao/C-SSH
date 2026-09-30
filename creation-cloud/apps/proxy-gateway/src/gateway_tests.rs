//! 验证 SSH 兼容、RDP 完整首帧转发及首字节与协商等待期间的租约取消。

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use chrono::{TimeDelta, Utc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, duplex},
    sync::{oneshot, watch},
    time,
};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{authorize::Client, config::AuthorizationEndpoint, connect_request::TargetProtocol};

use super::{
    CONNECTED, GatewayError, InitialData, deliver_and_transfer, monitor_authorization,
    run_guarded_data,
};

const RDP_REQUEST: &[u8] = &[3, 0, 0, 19, 14, 0xe0, 0, 0, 0, 0, 0, 1, 0, 8, 0, 3, 0, 0, 0];
const RDP_CONFIRM: &[u8] = &[
    3, 0, 0, 19, 14, 0xd0, 0, 0, 0x12, 0x34, 0, 2, 0x0f, 8, 0, 2, 0, 0, 0,
];

fn initial(protocol: TargetProtocol, prefetched: &[u8]) -> InitialData<'_> {
    InitialData {
        server_banner: if protocol == TargetProtocol::Ssh {
            b"SSH-2.0-server\r\n"
        } else {
            b""
        },
        prefetched,
        protocol,
        handshake_timeout: Duration::from_secs(1),
    }
}

#[tokio::test]
async fn failed_guard_wins_before_first_data_byte() {
    let data_started = AtomicBool::new(false);
    let data = async {
        data_started.store(true, Ordering::SeqCst);
        Ok(())
    };
    let monitor = async { GatewayError::AuthorizationDenied };
    let (_shutdown_sender, mut shutdown) = watch::channel(false);
    let result = run_guarded_data(data, monitor, &mut shutdown).await;
    assert!(matches!(result, Err(GatewayError::AuthorizationDenied)));
    assert!(!data_started.load(Ordering::SeqCst));
}

#[tokio::test]
async fn ssh_delivers_original_banner_prefetch_and_bidirectional_tail() {
    let (mut client, mut client_peer) = duplex(512);
    let (mut target, mut target_peer) = duplex(512);
    let data = deliver_and_transfer(
        &mut client,
        &mut target,
        initial(TargetProtocol::Ssh, b"SSH-2.0-client\r\n"),
    );
    let peers = async {
        let expected = [CONNECTED, b"SSH-2.0-server\r\n"].concat();
        let mut header = vec![0; expected.len()];
        client_peer.read_exact(&mut header).await.unwrap();
        assert_eq!(header, expected);
        client_peer.write_all(b"client-tail").await.unwrap();
        client_peer.shutdown().await.unwrap();
        let mut received = vec![0; b"SSH-2.0-client\r\nclient-tail".len()];
        target_peer.read_exact(&mut received).await.unwrap();
        assert_eq!(&received, b"SSH-2.0-client\r\nclient-tail");
        target_peer.write_all(b"server-tail").await.unwrap();
        target_peer.shutdown().await.unwrap();
        let mut tail = Vec::new();
        client_peer.read_to_end(&mut tail).await.unwrap();
        assert_eq!(tail, b"server-tail");
    };
    let (result, ()) = time::timeout(Duration::from_secs(2), async { tokio::join!(data, peers) })
        .await
        .unwrap();
    result.expect("SSH 原有转发应成功");
}

#[tokio::test]
async fn rdp_client_first_handshake_preserves_fragmented_prefetch_and_relay() {
    let (mut client, mut client_peer) = duplex(512);
    let (mut target, mut target_peer) = duplex(512);
    let data = deliver_and_transfer(
        &mut client,
        &mut target,
        initial(TargetProtocol::Rdp, &RDP_REQUEST[..6]),
    );
    let peers = async {
        let mut header = vec![0; CONNECTED.len()];
        client_peer.read_exact(&mut header).await.unwrap();
        assert_eq!(header, CONNECTED);
        client_peer
            .write_all(&[&RDP_REQUEST[6..], b"client-tls"].concat())
            .await
            .unwrap();
        let mut request = vec![0; RDP_REQUEST.len()];
        target_peer.read_exact(&mut request).await.unwrap();
        assert_eq!(request, RDP_REQUEST);
        target_peer
            .write_all(&[RDP_CONFIRM, b"server-tls"].concat())
            .await
            .unwrap();
        target_peer.shutdown().await.unwrap();
        let mut received = vec![0; RDP_CONFIRM.len() + 10];
        client_peer.read_exact(&mut received).await.unwrap();
        assert_eq!(received, [RDP_CONFIRM, b"server-tls"].concat());
        client_peer.shutdown().await.unwrap();
        let mut received = Vec::new();
        target_peer.read_to_end(&mut received).await.unwrap();
        assert_eq!(received, b"client-tls");
    };
    let (result, ()) = time::timeout(Duration::from_secs(2), async { tokio::join!(data, peers) })
        .await
        .unwrap();
    result.expect("RDP 双向协商及后续转发应成功");
}

#[tokio::test]
async fn lease_expiry_closes_rdp_waiting_for_client_request_after_200() {
    let (mut client, mut client_peer) = duplex(512);
    let (mut target, mut target_peer) = duplex(512);
    let (_shutdown_sender, mut shutdown) = watch::channel(false);
    let (observed, ready) = oneshot::channel();
    let authorization = Client::new(
        AuthorizationEndpoint {
            address: "127.0.0.1:0".parse().unwrap(),
            host_header: "unused".to_owned(),
            authorization_path: "/unused",
            close_path: "/unused",
        },
        Arc::new(Zeroizing::new("unused".to_owned())),
        Duration::from_secs(1),
    );
    let monitor = async {
        ready.await.unwrap();
        monitor_authorization(
            &authorization,
            Uuid::new_v4(),
            Utc::now() + TimeDelta::milliseconds(20),
            Duration::from_secs(1),
        )
        .await
    };
    let guarded = run_guarded_data(
        deliver_and_transfer(&mut client, &mut target, initial(TargetProtocol::Rdp, b"")),
        monitor,
        &mut shutdown,
    );
    let peer = async {
        let mut header = vec![0; CONNECTED.len()];
        client_peer.read_exact(&mut header).await.unwrap();
        assert_eq!(header, CONNECTED);
        observed.send(()).unwrap();
    };
    let (result, ()) = time::timeout(Duration::from_secs(2), async {
        tokio::join!(guarded, peer)
    })
    .await
    .unwrap();
    assert!(matches!(result, Err(GatewayError::LeaseExpired)));
    drop(target);
    let mut forwarded = Vec::new();
    target_peer.read_to_end(&mut forwarded).await.unwrap();
    assert!(forwarded.is_empty());
}

#[tokio::test]
async fn revocation_closes_rdp_waiting_for_server_confirm_without_releasing_tail() {
    let (mut client, mut client_peer) = duplex(512);
    let (mut target, mut target_peer) = duplex(512);
    let (_shutdown_sender, mut shutdown) = watch::channel(false);
    let (observed, ready) = oneshot::channel();
    let monitor = async {
        ready.await.unwrap();
        GatewayError::AuthorizationDenied
    };
    let prefetched = [RDP_REQUEST, b"must-not-forward"].concat();
    let guarded = run_guarded_data(
        deliver_and_transfer(
            &mut client,
            &mut target,
            initial(TargetProtocol::Rdp, &prefetched),
        ),
        monitor,
        &mut shutdown,
    );
    let peer = async {
        let mut request = vec![0; RDP_REQUEST.len()];
        target_peer.read_exact(&mut request).await.unwrap();
        assert_eq!(request, RDP_REQUEST);
        observed.send(()).unwrap();
    };
    let (result, ()) = time::timeout(Duration::from_secs(2), async {
        tokio::join!(guarded, peer)
    })
    .await
    .unwrap();
    assert!(matches!(result, Err(GatewayError::AuthorizationDenied)));
    drop(client);
    drop(target);
    let mut received = Vec::new();
    client_peer.read_to_end(&mut received).await.unwrap();
    assert_eq!(received, CONNECTED);
    let mut tail = Vec::new();
    target_peer.read_to_end(&mut tail).await.unwrap();
    assert!(tail.is_empty());
}
