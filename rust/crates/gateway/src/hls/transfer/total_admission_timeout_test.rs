use super::test_fixture::fixture;
use super::HlsTransfer;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn occupied_gate_expires_as_the_hls_total_deadline() {
    let (executor, url, server) = fixture().await;
    let held = executor
        .get(&url, PreemptionAuthority::Transition)
        .expect("request")
        .admit()
        .await
        .expect("held lease");
    let request = executor
        .get(&url, PreemptionAuthority::PlaybackCritical)
        .expect("request");
    let timing = HlsTransferTimeouts::new(
        Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_millis(20),
    );
    let Err(error) = HlsTransfer::open(request, timing).await else {
        panic!("transfer must hit its total deadline")
    };
    assert!(error.to_string().contains("object transfer timed out"));
    drop(held);
    server.abort();
}
