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
async fn newly_advertised_root_does_not_inherit_a_failed_roots_backoff() {
    let failed = HlsScript::new("root", [StatusCode::SERVICE_UNAVAILABLE]);
    let healthy = HlsScript::new("never", []);
    let failed_url = serve(failed.clone()).await;
    let healthy_url = serve(healthy.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.base = Duration::from_secs(5);
    options.tuning.retry.max = Duration::from_secs(5);
    let harness = start_harness("hls-new-root-during-backoff", options);

    harness.handle.update_focus(focus(&[&failed_url]));
    failed.wait_for_hits(1).await;
    wait_queued(&harness).await;
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(failed.paths(), vec!["root"]);
    harness
        .handle
        .update_focus(focus(&[&failed_url, &healthy_url]));
    healthy.wait_for_hits(1).await;
    let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

    assert_eq!(terminal.phase, SegmentedPhase::Ready);
    assert_eq!(failed.paths(), vec!["root"]);
    assert_eq!(healthy.paths(), vec!["root", "child", "init", "segment"]);
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_queued(harness: &delivery_fixture::DeliveryHarness) {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        while harness.segmented.snapshot("stream").phase != SegmentedPhase::Queued {
            changed.notified().await;
        }
    })
    .await
    .expect("failed root enters strict cooldown");
}

fn focus(sources: &[&str]) -> ghostr_delivery::delivery_events::DeliveryFocus {
    let mut item = sized_item("stream", sources[0], 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls = sources.iter().map(|source| (*source).to_owned()).collect();
    focus_now(vec![item], 0, 0)
}
