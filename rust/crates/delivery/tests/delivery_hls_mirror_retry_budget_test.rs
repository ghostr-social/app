mod delivery_fixture;
mod hls_terminal_wait;

use axum::http::StatusCode;
use delivery_fixture::hls_recovery::{serve, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn successful_bootstrap_prefixes_cannot_erase_downstream_failure_budgets() {
    let failures = [StatusCode::SERVICE_UNAVAILABLE; 2];
    let primary = HlsScript::new("init", failures);
    let mirror = HlsScript::new("init", failures);
    let primary_url = serve(primary.clone()).await;
    let mirror_url = serve(mirror.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 2;
    let harness = start_harness("hls-mirror-retry-budget", options);
    let mut item = sized_item("stream", &primary_url, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls.push(mirror_url);

    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

    assert_eq!(terminal.phase, SegmentedPhase::Failed);
    assert_eq!(init_hits(&primary), 2);
    assert_eq!(init_hits(&mirror), 2);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn init_hits(script: &HlsScript) -> usize {
    script
        .paths()
        .iter()
        .filter(|path| **path == "init")
        .count()
}
