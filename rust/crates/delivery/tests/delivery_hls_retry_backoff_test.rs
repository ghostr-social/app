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
async fn hls_retry_waits_for_its_timer_without_unrelated_events() {
    let script = HlsScript::new("init", [StatusCode::SERVICE_UNAVAILABLE]);
    let source = serve(script.clone()).await;
    let harness = start_harness("hls-retry-backoff", DeliveryOptions::default());
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    script.wait_for_hits(3).await;
    let first = script.hits()[2].at;

    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(
        script.hits().len(),
        3,
        "retry started before its 50 ms floor"
    );
    script.wait_for_hits(4).await;
    assert!(script.hits()[3].at.duration_since(first) >= Duration::from_millis(45));

    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;
    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    std::fs::remove_dir_all(&harness.root).ok();
}
