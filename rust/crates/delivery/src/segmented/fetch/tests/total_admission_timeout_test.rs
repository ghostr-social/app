use super::super::telemetry::FetchProgress;
use super::super::{open, FetchRuntime, FetchSpec};
use super::support::{client, network_status, stalled_headers};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn occupied_gate_expires_as_the_hls_total_deadline() {
    let (url, server) = stalled_headers().await;
    let requests = client();
    let held = requests
        .get(&url, PreemptionAuthority::Transition)
        .expect("request")
        .admit()
        .await
        .expect("held lease");
    let timing = HlsTransferTimeouts::new(
        Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_millis(20),
    );
    let deadline = tokio::time::Instant::now() + timing.total;
    let network = network_status();
    let progress = FetchProgress::default();
    let runtime = FetchRuntime::new(&requests, deadline, &network, &progress);
    let error = match open(runtime, spec(&url, timing)).await {
        Ok(_) => panic!("transfer must hit its total deadline"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("object transfer timed out"));
    drop(held);
    server.abort();
}

fn spec(url: &str, timeouts: HlsTransferTimeouts) -> FetchSpec<'_> {
    FetchSpec {
        url,
        limit: 1,
        require_manifest: false,
        timeouts,
        priority: PreemptionAuthority::Transition,
        admission_fence: None,
    }
}
