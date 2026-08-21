use super::super::{open, FetchSpec};
use super::support::{client, stalled_headers};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn header_deadline_wins_before_the_hls_total_deadline() {
    let (url, server) = stalled_headers().await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(20),
        Duration::from_secs(1),
        Duration::from_secs(1),
    );
    let error = match open(&client(), spec(&url, timing), deadline(timing)).await {
        Ok(_) => panic!("transfer must hit its header deadline"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("response headers timed out"));
    assert!(!error.to_string().contains("object transfer timed out"));
    server.abort();
}

fn spec(url: &str, timeouts: HlsTransferTimeouts) -> FetchSpec<'_> {
    FetchSpec {
        url,
        limit: 1,
        require_manifest: false,
        timeouts,
        priority: PreemptionAuthority::Transition,
    }
}

fn deadline(timeouts: HlsTransferTimeouts) -> tokio::time::Instant {
    tokio::time::Instant::now() + timeouts.total
}
