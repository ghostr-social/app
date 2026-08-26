mod delivery_fixture;
mod hls_terminal_wait;

use axum::http::StatusCode;
use core::time::Duration;
use delivery_fixture::hls::{serve, HlsGate};
use delivery_fixture::hls_recovery::{serve as serve_script, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn focus_regeneration_does_not_reseed_a_retired_hls_root() {
    let primary = HlsScript::new("root", [StatusCode::NOT_FOUND, StatusCode::NOT_FOUND]);
    let primary_url = serve_script(primary.clone()).await;
    let backup = HlsGate::new();
    let backup_url = serve(backup.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.permanent_attempts = 1;
    options.tuning.retry.revive_after = Duration::from_secs(5);
    let harness = start_harness("hls-retired-root-focus", options);

    harness
        .handle
        .update_focus(focus(&primary_url, &backup_url, 0));
    wait_backup_root(&backup).await;
    harness
        .handle
        .update_focus(focus(&primary_url, &backup_url, 1));
    backup.release.add_permits(1);
    tokio::time::sleep(Duration::from_millis(30)).await;
    harness
        .handle
        .update_focus(focus(&primary_url, &backup_url, 0));
    wait_backup_root(&backup).await;
    backup.release.add_permits(1);

    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;
    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    assert_eq!(primary.paths(), vec!["root"]);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn focus(
    primary: &str,
    backup: &str,
    current: usize,
) -> ghostr_delivery::delivery_events::DeliveryFocus {
    let mut item = sized_item("stream", primary, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls.push(backup.to_owned());
    let other = sized_item("other", "https://unused.test/video.mp4", 32, 4_000);
    focus_now(vec![item, other], current, 0)
}

async fn wait_backup_root(gate: &HlsGate) {
    tokio::time::timeout(Duration::from_secs(2), gate.started.acquire())
        .await
        .expect("backup root request")
        .expect("backup gate")
        .forget();
}
