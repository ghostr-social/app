use super::asset_with_timeouts;
use super::support::{client, trickled_body};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn trickled_hls_chunks_cannot_extend_total_deadline() {
    let (url, server) = trickled_body(Duration::from_millis(5)).await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(100),
        Duration::from_millis(20),
        Duration::from_millis(30),
    );

    let requests = client();
    let Err(error) =
        asset_with_timeouts(&requests, &url, timing, PreemptionAuthority::Transition).await
    else {
        panic!("transfer must hit its total deadline")
    };

    assert!(error.to_string().contains("transfer timed out"));
    assert_eq!(error.reason(), ErrorReason::Timeout);
    let actual = error.actual_resources().expect("admitted request usage");
    assert!(actual.network_bytes > 0);
    assert_eq!(actual.network_bytes, error.network_bytes());
    assert_eq!(actual.storage_bytes, 0);
    assert_eq!(actual.requests, 1);
    server.abort();
}
