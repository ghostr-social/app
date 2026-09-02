#[path = "watcher_progressive_eta_test/fixture.rs"]
mod fixture;
#[path = "watcher_progressive_eta_test/plan.rs"]
mod plan;

use crate::api::delivery_types::FfiDeliveryEventKind;

#[tokio::test]
async fn causal_plan_change_streams_progressive_playback_eta() {
    let mut watcher = fixture::ProgressiveEtaWatcher::start().await;

    let baseline = watcher.next().await;
    assert_eq!(baseline.kind, FfiDeliveryEventKind::Readiness);
    assert_eq!(baseline.eta_ms, None, "stale focus plan must not leak");

    watcher.publish_causal_plan(175);
    let planned = watcher.next().await;
    assert_eq!(planned.kind, FfiDeliveryEventKind::Progress);
    assert_eq!(
        planned.eta_ms,
        Some(200),
        "ETA is rounded up to 100 ms buckets"
    );
}

#[tokio::test]
async fn eta_drift_inside_a_bucket_streams_nothing() {
    let mut watcher = fixture::ProgressiveEtaWatcher::start().await;
    watcher.next().await;
    watcher.publish_causal_plan(175);
    assert_eq!(watcher.next().await.eta_ms, Some(200));

    watcher.publish_causal_plan(190);
    watcher.expect_quiet().await;
}
