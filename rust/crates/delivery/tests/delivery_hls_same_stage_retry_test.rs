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
async fn transient_initialization_failure_retries_the_same_stage() {
    let script = HlsScript::new("init", [StatusCode::SERVICE_UNAVAILABLE]);
    let source = serve(script.clone()).await;
    let harness = start_harness("hls-same-stage-retry", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));

    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    assert_eq!(
        script.paths(),
        vec!["root", "child", "init", "init", "segment"]
    );
    std::fs::remove_dir_all(&harness.root).ok();
}
