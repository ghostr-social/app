use super::super::{fetch_stage, StagedFetch};
use super::support::{client, immediate_asset, network_status};
use crate::manager::time::unix_time_ms;
use ghostr_engine::adaptive::{HlsBootstrapStage, PreemptionAuthority, ResourceCost};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn cancellation_after_admission_keeps_exact_partial_network_usage() {
    let (url, sent, server) = partial_asset().await;
    let requests = client();
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let network = network_status();
        fetch_stage(input(&requests, &network, &url, cancelled)).await
    });
    sent.await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    cancel.send(()).unwrap();
    let failure = task.await.unwrap().err().expect("cancelled fetch");

    assert!(failure.is_cancelled());
    assert_eq!(failure.network_bytes(), 37);
    assert_eq!(
        failure.actual_resources(),
        Some(ResourceCost::new(37, 0, 0, 1))
    );
    server.abort();
}

#[tokio::test]
async fn cancellation_before_admission_has_no_request_or_byte_usage() {
    let (url, server) = immediate_asset().await;
    let requests = client();
    let held = requests
        .get(&url, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap();
    let (cancel, cancelled) = tokio::sync::oneshot::channel();
    let queued = requests.clone();
    let task = tokio::spawn(async move {
        let network = network_status();
        fetch_stage(input(&queued, &network, &url, cancelled)).await
    });
    tokio::task::yield_now().await;
    cancel.send(()).unwrap();
    let failure = task.await.unwrap().err().expect("cancelled fetch");

    assert!(failure.is_cancelled());
    assert!(failure.origin().is_none());
    assert!(failure.actual_resources().is_none());
    drop(held);
    server.abort();
}

fn input<'a>(
    requests: &'a ghostr_net::media_request_executor::MediaRequestExecutor,
    network_status: &'a crate::delivery_events::DeliveryNetworkStatusReader,
    url: &'a str,
    cancellation: tokio::sync::oneshot::Receiver<()>,
) -> StagedFetch<'a> {
    StagedFetch {
        requests,
        stage: HlsBootstrapStage::FirstSegment,
        url,
        priority: PreemptionAuthority::PlaybackCritical,
        committed_until_ms: unix_time_ms() + 1_000,
        network_status,
        cancellation: Some(cancellation),
        traffic: None,
    }
}

async fn partial_asset() -> (
    String,
    tokio::sync::oneshot::Receiver<()>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent, received) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        let read = socket.read(&mut request).await.unwrap();
        assert!(read > 0);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n1234567890123456789012345678901234567")
            .await
            .unwrap();
        sent.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    (format!("http://{address}/segment.m4s"), received, server)
}
