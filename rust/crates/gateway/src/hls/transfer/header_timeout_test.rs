use super::test_fixture::fixture;
use super::HlsTransfer;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn header_deadline_wins_before_the_hls_total_deadline() {
    let (executor, url, server) = fixture().await;
    let request = executor
        .get(&url, PreemptionAuthority::PlaybackCritical)
        .expect("request");
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(20),
        Duration::from_secs(1),
        Duration::from_secs(1),
    );
    let Err(error) = HlsTransfer::open(request, timing).await else {
        panic!("transfer must hit its header deadline")
    };
    assert!(error.to_string().contains("response headers timed out"));
    assert!(!error.to_string().contains("object transfer timed out"));
    server.abort();
}
