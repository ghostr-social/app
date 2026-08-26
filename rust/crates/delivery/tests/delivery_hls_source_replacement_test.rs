mod delivery_fixture;
mod hls_terminal_wait;

use axum::http::StatusCode;
use core::time::Duration;
use delivery_fixture::hls_recovery::{serve, HlsScript};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn replacement_hls_roots_clear_the_previous_strict_cooldown() {
    let failed = HlsScript::new("root", [StatusCode::SERVICE_UNAVAILABLE]);
    let replacement = HlsScript::new("never", []);
    let failed_url = serve(failed.clone()).await;
    let replacement_url = serve(replacement.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.base = Duration::from_secs(5);
    options.tuning.retry.max = Duration::from_secs(5);
    let harness = start_harness("hls-source-replacement", options);

    harness.handle.update_focus(focus(&failed_url));
    failed.wait_for_hits(1).await;
    wait_queued(&harness).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(
        failed.paths(),
        vec!["root"],
        "strict cooldown was not installed"
    );
    harness.handle.update_focus(focus(&replacement_url));
    replacement.wait_for_hits(1).await;
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    assert_eq!(failed.paths(), vec!["root"]);
    assert_eq!(
        replacement.paths(),
        vec!["root", "child", "init", "segment"]
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

fn focus(source: &str) -> ghostr_delivery::delivery_events::DeliveryFocus {
    let mut item = sized_item("stream", source, 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    focus_now(vec![item], 0, 0)
}

async fn wait_queued(harness: &delivery_fixture::DeliveryHarness) {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if harness.segmented.snapshot("stream").phase == SegmentedPhase::Queued {
                return;
            }
            notified.await;
        }
    })
    .await
    .expect("failed root enters strict cooldown");
}
