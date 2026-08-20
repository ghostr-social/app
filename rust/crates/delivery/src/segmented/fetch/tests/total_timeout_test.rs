use super::super::asset_with_timeouts;
use super::support::{client, trickled_body};
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn trickled_hls_chunks_cannot_extend_total_deadline() {
    let (url, server) = trickled_body(Duration::from_millis(5)).await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(100),
        Duration::from_millis(20),
        Duration::from_millis(30),
    );

    let error = match asset_with_timeouts(client().as_ref(), &url, timing).await {
        Ok(_) => panic!("transfer must hit its total deadline"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("transfer timed out"));
    server.abort();
}
