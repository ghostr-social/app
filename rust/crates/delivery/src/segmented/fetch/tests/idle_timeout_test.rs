use super::asset_with_timeouts;
use super::support::{client, stalled_body};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn stalled_hls_chunk_hits_injected_idle_deadline() {
    let (url, server) = stalled_body().await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(100),
        Duration::from_millis(10),
        Duration::from_millis(100),
    );

    let requests = client();
    let Err(error) =
        asset_with_timeouts(&requests, &url, timing, PreemptionAuthority::Transition).await
    else {
        panic!("stalled chunk must time out")
    };

    assert!(error.to_string().contains("body idle timed out"));
    server.abort();
}
