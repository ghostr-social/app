mod delivery_fixture;

use axum::http::StatusCode;
use delivery_fixture::hls_recovery::{serve, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use std::time::Duration;

#[tokio::test]
async fn fully_retired_hls_source_revives_from_an_exact_timer_wake() {
    let script = HlsScript::new("root", [StatusCode::SERVICE_UNAVAILABLE]);
    let source = serve(script.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    options.tuning.retry.revive_after = Duration::from_millis(80);
    let harness = start_harness("hls-retirement-revival", options);
    let mut item = sized_item("stream", &source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    script.wait_for_hits(1).await;
    wait_phase(&harness, SegmentedPhase::Failed).await;

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        script.hits().len(),
        1,
        "retired root retried before revival"
    );
    wait_phase(&harness, SegmentedPhase::Ready).await;

    assert_eq!(
        script.paths(),
        vec!["root", "root", "child", "init", "segment"]
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_phase(harness: &delivery_fixture::DeliveryHarness, expected: SegmentedPhase) {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if harness.segmented.snapshot("stream").phase == expected {
                return;
            }
            notified.await;
        }
    })
    .await
    .expect("HLS phase transition");
}
