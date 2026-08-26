use super::test_fixture::fixture;
use super::HlsTransfer;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn total_deadline_wins_while_waiting_for_hls_headers() {
    let (executor, url, server) = fixture().await;
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
    assert!(error.to_string().contains("transfer timed out"));
    assert!(!error.to_string().contains("response headers timed out"));
    server.abort();
}
