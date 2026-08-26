mod delivery_fixture;
mod hls_terminal_wait;

use core::time::Duration;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::adaptive::{
    DecisionOutcome, HlsBootstrapStage, RecordedHlsBootstrapStage, RecordedWarpCommand,
};
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn hls_bootstrap_objects_are_singular_recorded_warp_commitments() {
    let gate = HlsGate::new();
    let source = serve(gate.clone()).await;
    let harness = start_harness("hls-warp-decisions", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    tokio::time::timeout(Duration::from_secs(2), gate.started.acquire())
        .await
        .expect("selected root request starts")
        .expect("gate remains open")
        .forget();
    let pending = hls_records(&harness, Some(HlsBootstrapStage::RootManifest));
    assert_eq!(pending.len(), 1);
    assert!(pending[0].chosen_action_id.is_some());
    assert_eq!(pending[0].eventual_outcome, DecisionOutcome::Pending);

    gate.release.add_permits(1);
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;
    assert_eq!(
        terminal.phase,
        ghostr_delivery::segmented::SegmentedPhase::Ready
    );
    let completed = hls_records(&harness, None);
    let stages: Vec<_> = completed.iter().filter_map(recorded_stage).collect();
    assert_eq!(
        stages,
        vec![
            RecordedHlsBootstrapStage::RootManifest,
            RecordedHlsBootstrapStage::ChildPlaylist,
            RecordedHlsBootstrapStage::Initialization,
            RecordedHlsBootstrapStage::FirstSegment,
        ]
    );
    assert!(completed.iter().all(|record| {
        record.chosen_action_id.is_some()
            && matches!(record.eventual_outcome, DecisionOutcome::Succeeded { bytes, .. } if exact_actual(record, bytes))
    }));
    assert_eq!(gate.hits(), vec!["root", "child", "init", "segment"]);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn exact_actual(record: &ghostr_engine::adaptive::DecisionRecord, bytes: u64) -> bool {
    record.actual_resources.is_some_and(|actual| {
        actual.network_bytes == bytes
            && actual.storage_bytes == bytes
            && actual.cpu_ms == 0
            && actual.requests == 1
    })
}

fn hls_records(
    harness: &delivery_fixture::DeliveryHarness,
    stage: Option<HlsBootstrapStage>,
) -> Vec<ghostr_engine::adaptive::DecisionRecord> {
    harness
        .handle
        .decision_history()
        .records
        .into_iter()
        .filter(|record| {
            recorded_stage(record)
                .is_some_and(|value| stage.is_none_or(|wanted| value == wanted.into()))
        })
        .collect()
}

fn recorded_stage(
    record: &ghostr_engine::adaptive::DecisionRecord,
) -> Option<RecordedHlsBootstrapStage> {
    let command = &record.warp_decision.as_ref()?.selected.as_ref()?.command;
    match command {
        RecordedWarpCommand::FetchHlsBootstrap { stage, .. } => Some(*stage),
        _ => None,
    }
}
