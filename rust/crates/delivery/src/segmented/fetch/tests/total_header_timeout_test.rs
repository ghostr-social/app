use super::super::axiom_test_support::open;
use super::super::telemetry::FetchProgress;
use super::super::{FetchRuntime, FetchSpec};
use super::support::{client, network_status, stalled_headers};
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn total_deadline_wins_while_waiting_for_hls_headers() {
    let (url, server) = stalled_headers().await;
    let timing = HlsTransferTimeouts::new(
        Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_millis(20),
    );
    let deadline = tokio::time::Instant::now() + timing.total;
    let spec = FetchSpec {
        url: &url,
        limit: 1,
        object_limit: 1,
        object: Default::default(),
        timeouts: timing,
        priority: PreemptionAuthority::Transition,
        admission_fence: None,
    };
    let requests = client();
    let network = network_status();
    let progress = FetchProgress::default();
    let runtime = FetchRuntime::new(&requests, deadline, &network, &progress);
    let result = open(runtime, spec).await;
    let Err(error) = result else {
        panic!("transfer must hit its total deadline")
    };
    assert!(error.to_string().contains("transfer timed out"));
    assert!(!error.to_string().contains("response headers timed out"));
    server.abort();
}
