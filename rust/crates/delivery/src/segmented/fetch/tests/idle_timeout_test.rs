use super::super::asset_with_timeouts;
use super::support::{client, stalled_body};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn stalled_hls_chunk_hits_injected_idle_deadline() {
    let (url, server) = stalled_body().await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(100),
        Duration::from_millis(10),
        Duration::from_millis(100),
    );

    let requests = client();
    let error =
        match asset_with_timeouts(&requests, &url, timing, PreemptionAuthority::Transition).await {
            Ok(_) => panic!("stalled chunk must time out"),
            Err(error) => error,
        };

    assert!(error.to_string().contains("body idle timed out"));
    server.abort();
}
