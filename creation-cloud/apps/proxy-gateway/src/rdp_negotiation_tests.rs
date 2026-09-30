//! 验证 RDP 协商的字节边界、双向原样性和失败后不开放转发。

use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadBuf, duplex};

use super::{NegotiationError, negotiate, read_packet, validate_confirm, validate_request};

fn packet(kind: u8, body: &[u8]) -> Vec<u8> {
    let length = 11 + body.len();
    let mut bytes = vec![3, 0, 0, u8::try_from(length).expect("夹具必须有界")];
    bytes.extend_from_slice(&[u8::try_from(length - 5).unwrap(), kind, 0, 0, 0x12, 0x34, 0]);
    bytes.extend_from_slice(body);
    bytes
}

fn request() -> Vec<u8> {
    packet(
        0xe0,
        &[
            b"Cookie: mstshash=example\r\n".as_slice(),
            &[1, 0, 8, 0, 3, 0, 0, 0],
        ]
        .concat(),
    )
}

fn confirm() -> Vec<u8> {
    packet(0xd0, &[2, 0x0f, 8, 0, 2, 0, 0, 0])
}

#[test]
fn recognizes_rdp_request_and_confirm_with_nonzero_x224_reference() {
    assert_eq!(validate_request(&request()).unwrap(), 3);
    assert!(validate_confirm(&confirm(), 3).unwrap());
    assert!(!validate_confirm(&packet(0xd0, &[3, 0, 8, 0, 5, 0, 0, 0]), 3).unwrap());
    let mut correlation = vec![1, 8, 8, 0, 3, 0, 0, 0, 6, 0, 36, 0];
    correlation.extend_from_slice(&[0x42; 16]);
    correlation.extend_from_slice(&[0; 16]);
    assert_eq!(validate_request(&packet(0xe0, &correlation)).unwrap(), 3);
}

#[test]
fn rejects_wrong_tpkt_x224_lengths_and_protocols() {
    let valid = request();
    for (index, value) in [(0, 4), (1, 1), (3, 11), (4, 255), (5, 0xd0), (10, 1)] {
        let mut invalid = valid.clone();
        invalid[index] = value;
        assert!(validate_request(&invalid).is_err());
    }
    for body in [
        vec![],
        vec![1, 0, 8, 0, 32, 0, 0, 0],
        vec![1, 0, 9, 0, 3, 0, 0, 0],
        vec![1, 4, 8, 0, 3, 0, 0, 0],
        vec![1, 8, 8, 0, 3, 0, 0, 0],
        vec![1, 0, 8, 0, 3, 0, 0, 0, 0],
        b"GET / HTTP/1.1\r\n\r\n".to_vec(),
    ] {
        assert!(validate_request(&packet(0xe0, &body)).is_err());
    }
    assert!(validate_confirm(&confirm(), 1).is_err());
    for body in [
        vec![],
        vec![2, 0, 8, 0, 3, 0, 0, 0],
        vec![2, 0, 8, 0, 32, 0, 0, 0],
        vec![3, 0, 8, 0, 7, 0, 0, 0],
        vec![3, 1, 8, 0, 5, 0, 0, 0],
        vec![1, 0, 8, 0, 2, 0, 0, 0],
    ] {
        assert!(validate_confirm(&packet(0xd0, &body), 3).is_err());
    }
}

#[tokio::test]
async fn rejects_overlong_and_truncated_input_before_allocation_or_forwarding() {
    for input in [
        vec![3, 0, 1, 4],
        vec![3, 0, 0, 19, 14, 0xe0],
        b"SSH-2.0-test\r\n".to_vec(),
    ] {
        assert!(read_packet(&mut input.as_slice()).await.is_err());
    }
    let (mut client, _client_peer) = duplex(64);
    let (mut target, mut target_peer) = duplex(64);
    let invalid = b"GET / HTTP/1.1\r\n\r\n";
    assert!(negotiate(&mut client, &mut target, invalid).await.is_err());
    let mut byte = [0];
    let mut buffer = ReadBuf::new(&mut byte);
    assert!(matches!(
        Pin::new(&mut target_peer).poll_read(&mut Context::from_waker(Waker::noop()), &mut buffer),
        Poll::Pending
    ));
}

#[tokio::test]
async fn forwards_negotiation_exactly_and_holds_prefetch_until_confirm() {
    let sent = request();
    let expected = sent.clone();
    let accepted = confirm();
    let expected_confirm = accepted.clone();
    let prefetched = [sent, b"client-tls".to_vec()].concat();
    let (mut client, mut client_peer) = duplex(512);
    let (mut target, mut target_peer) = duplex(512);
    let forward = negotiate(&mut client, &mut target, &prefetched);
    let peer = async {
        let mut received = vec![0; expected.len()];
        target_peer.read_exact(&mut received).await.unwrap();
        assert_eq!(received, expected);
        let mut byte = [0];
        let mut buffer = ReadBuf::new(&mut byte);
        assert!(matches!(
            Pin::new(&mut target_peer)
                .poll_read(&mut Context::from_waker(Waker::noop()), &mut buffer),
            Poll::Pending
        ));
        target_peer
            .write_all(&[accepted, b"server-tls".to_vec()].concat())
            .await
            .unwrap();
        let mut received_confirm = vec![0; expected_confirm.len()];
        client_peer.read_exact(&mut received_confirm).await.unwrap();
        assert_eq!(received_confirm, expected_confirm);
        let mut tail = [0; 10];
        target_peer.read_exact(&mut tail).await.unwrap();
        assert_eq!(&tail, b"client-tls");
    };
    let (result, ()) = tokio::join!(forward, peer);
    result.expect("双方协商应成功");
    let mut tail = [0; 10];
    target.read_exact(&mut tail).await.unwrap();
    assert_eq!(&tail, b"server-tls");
}

#[tokio::test]
async fn forwards_server_failure_without_releasing_client_prefetch() {
    let prefetched = [request(), b"must-not-forward".to_vec()].concat();
    let failure = packet(0xd0, &[3, 0, 8, 0, 5, 0, 0, 0]);
    let (mut client, mut client_peer) = duplex(512);
    let (mut target, mut target_peer) = duplex(512);
    target_peer.write_all(&failure).await.unwrap();
    assert!(matches!(
        negotiate(&mut client, &mut target, &prefetched).await,
        Err(NegotiationError::Refused)
    ));
    drop(target);
    let mut target_bytes = Vec::new();
    target_peer.read_to_end(&mut target_bytes).await.unwrap();
    assert_eq!(target_bytes, request());
    let mut received = vec![0; failure.len()];
    client_peer.read_exact(&mut received).await.unwrap();
    assert_eq!(received, failure);
}
