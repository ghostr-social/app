mod delivery_fixture;
mod hls_terminal_wait;

use axum::http::StatusCode;
use delivery_fixture::hls_recovery::{serve, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

#[tokio::test]
async fn ready_backup_survives_root_add_reorder_and_unrelated_removal() {
    let primary = HlsScript::new("root", [StatusCode::NOT_FOUND]);
    let backup = HlsScript::new("never", []);
    let third = HlsScript::new("never", []);
    let primary_url = serve(primary.clone()).await;
    let backup_url = serve(backup.clone()).await;
    let third_url = serve(third.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.permanent_attempts = 1;
    let harness = start_harness("hls-roster-reconciliation", options);

    harness
        .handle
        .update_focus(focus(&[&primary_url, &backup_url]));
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;
    assert_initial_ready(&primary, &backup, terminal.phase);

    reorder_roots(&harness, &backup_url, &primary_url, &third_url).await;

    assert_eq!(
        harness.segmented.snapshot("stream").phase,
        SegmentedPhase::Ready
    );
    assert_eq!(primary.paths(), vec!["root"]);
    assert_eq!(backup.paths(), vec!["root", "child", "init", "segment"]);
    assert!(third.paths().is_empty());
    std::fs::remove_dir_all(&harness.root).ok();
}

fn assert_initial_ready(primary: &HlsScript, backup: &HlsScript, phase: SegmentedPhase) {
    assert_eq!(phase, SegmentedPhase::Ready);
    assert_eq!(primary.paths(), vec!["root"]);
    assert_eq!(backup.paths(), vec!["root", "child", "init", "segment"]);
}

async fn reorder_roots(
    harness: &delivery_fixture::DeliveryHarness,
    backup: &str,
    primary: &str,
    third: &str,
) {
    harness
        .handle
        .update_focus(focus(&[backup, primary, third]));
    tokio::time::sleep(Duration::from_millis(75)).await;
    harness.handle.update_focus(focus(&[backup, primary]));
    tokio::time::sleep(Duration::from_millis(75)).await;
}

fn focus(sources: &[&str]) -> ghostr_delivery::delivery_events::DeliveryFocus {
    let mut item = sized_item("stream", sources[0], 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls = sources.iter().map(|source| (*source).to_owned()).collect();
    focus_now(vec![item], 0, 0)
}
