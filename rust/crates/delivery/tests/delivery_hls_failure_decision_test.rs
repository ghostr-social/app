mod delivery_fixture;
mod hls_terminal_wait;

use axum::http::StatusCode;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::hls_recovery::{serve, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::adaptive::{
    DecisionOutcome, DecisionRecord, RecordedHlsBootstrapStage, RecordedWarpCommand,
};
use ghostr_engine::DeliveryKind;
use std::collections::HashSet;

#[tokio::test]
async fn every_failed_hls_attempt_has_one_exact_terminal_decision() {
    let script = HlsScript::new("init", [StatusCode::SERVICE_UNAVAILABLE]);
    let source = serve(script).await;
    let harness = start_harness("hls-failure-decisions", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

    let records = hls_records(&harness.handle);
    assert_terminal_actions(&records);
    assert_failed_attempt(&records);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn hls_records(handle: &ghostr_delivery::delivery_events::DeliveryHandle) -> Vec<DecisionRecord> {
    handle
        .decision_history()
        .records
        .into_iter()
        .filter(|record| stage(record).is_some())
        .collect()
}

fn assert_terminal_actions(records: &[DecisionRecord]) {
    assert_eq!(
        records.len(),
        5,
        "every attempted HLS stage must be recorded"
    );
    let action_ids: HashSet<_> = records
        .iter()
        .map(|record| record.chosen_action_id.expect("bound HLS action"))
        .collect();
    assert_eq!(
        action_ids.len(),
        records.len(),
        "each HLS attempt must own a unique action"
    );
    assert_eq!(
        records.iter().filter_map(stage).collect::<Vec<_>>(),
        vec![
            RecordedHlsBootstrapStage::RootManifest,
            RecordedHlsBootstrapStage::ChildPlaylist,
            RecordedHlsBootstrapStage::Initialization,
            RecordedHlsBootstrapStage::Initialization,
            RecordedHlsBootstrapStage::FirstSegment,
        ],
        "the bootstrap and retry stages must be recorded in execution order"
    );
    assert!(
        records
            .iter()
            .all(|record| record.eventual_outcome != DecisionOutcome::Pending),
        "every recorded HLS action must be terminal"
    );
}

fn assert_failed_attempt(records: &[DecisionRecord]) {
    let failed: Vec<_> = records
        .iter()
        .filter(|record| matches!(record.eventual_outcome, DecisionOutcome::Failed { .. }))
        .collect();
    assert_eq!(
        failed.len(),
        1,
        "only the scripted initialization retry must fail"
    );
    assert!(
        matches!(
            &failed[0].eventual_outcome,
            DecisionOutcome::Failed { class, .. } if class == "warp_hls_http_5xx"
        ),
        "the failure must retain its HTTP 5xx class"
    );
    let actual = failed[0]
        .actual_resources
        .expect("failed attempt resources");
    assert_eq!(
        actual.network_bytes, 0,
        "the rejected response has no body bytes"
    );
    assert_eq!(
        actual.storage_bytes, 0,
        "the rejected response stores no bytes"
    );
    assert_eq!(
        actual.cpu_ms, 0,
        "the failed fetch performs no transform work"
    );
    assert_eq!(
        actual.requests, 1,
        "the failed action admitted exactly one request"
    );
}

fn stage(record: &ghostr_engine::adaptive::DecisionRecord) -> Option<RecordedHlsBootstrapStage> {
    match &record.warp_decision.as_ref()?.selected.as_ref()?.command {
        RecordedWarpCommand::FetchHlsBootstrap { stage, .. } => Some(*stage),
        _ => None,
    }
}
