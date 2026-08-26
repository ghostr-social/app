use super::super::axiom_test_support::open;
use super::super::telemetry::FetchProgress;
use super::super::{FetchRuntime, FetchSpec};
use super::support::{client, network_status, stalled_headers};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn header_deadline_wins_before_the_hls_total_deadline() {
    let (url, server) = stalled_headers().await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_millis(20),
        Duration::from_secs(1),
        Duration::from_secs(1),
    );
    let requests = client();
    let network = network_status();
    let progress = FetchProgress::default();
    let runtime = FetchRuntime::new(&requests, deadline(timing), &network, &progress);
    let Err(error) = open(runtime, spec(&url, timing)).await else {
        panic!("transfer must hit its header deadline")
    };
    assert!(error.to_string().contains("response headers timed out"));
    assert!(!error.to_string().contains("object transfer timed out"));
    server.abort();
}

fn spec(url: &str, timeouts: HlsTransferTimeouts) -> FetchSpec<'_> {
    FetchSpec {
        url,
        limit: 1,
        object_limit: 1,
        object: Default::default(),
        timeouts,
        priority: PreemptionAuthority::Transition,
        admission_fence: None,
    }
}

fn deadline(timeouts: HlsTransferTimeouts) -> tokio::time::Instant {
    tokio::time::Instant::now() + timeouts.total
}
