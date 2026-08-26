use super::super::{fetch_stage_tracked, FetchProgress, SegmentedTraffic, StagedFetch};
use super::support::{client, immediate_asset, network_status};
use crate::manager::traffic::{channel, TrafficEvent};
use ghostr_engine::adaptive::{HlsBootstrapStage, PreemptionAuthority};
use ghostr_engine::ActionId;

#[tokio::test]
async fn tracked_hls_traffic_closes_when_the_http_body_finishes() {
    let (url, server) = immediate_asset().await;
    let (events, _receiver) = tokio::sync::mpsc::unbounded_channel();
    let (publisher, inbox) = channel(events, 8);
    let progress = FetchProgress::new(Some(SegmentedTraffic::new(ActionId::new(17), publisher)));
    let requests = client();
    let network = network_status();
    let (_cancel, cancelled) = tokio::sync::oneshot::channel();
    let stage = HlsBootstrapStage::RootManifest;
    let fetched = fetch_stage_tracked(
        StagedFetch {
            requests: &requests,
            stage,
            url: &url,
            maximum_bytes: stage.block_bytes(128 * 1024),
            continuation: None,
            priority: PreemptionAuthority::Transition,
            committed_until_ms: crate::manager::time::unix_time_ms() + 10_000,
            network_status: &network,
            cancellation: Some(cancelled),
            traffic: None,
        },
        &progress,
    )
    .await;

    assert!(fetched.result.is_ok());
    let batch = inbox.drain(tokio::time::Instant::now());
    assert!(matches!(
        batch.events().last(),
        Some(TrafficEvent::Closed { .. })
    ));
    server.await.expect("valid test fixture");
}
