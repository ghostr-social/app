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
async fn new_root_prepares_while_retired_roots_are_still_waiting() {
    let first = HlsScript::new("root", [StatusCode::NOT_FOUND]);
    let second = HlsScript::new("root", [StatusCode::NOT_FOUND]);
    let third = HlsScript::new("never", []);
    let first_url = serve(first.clone()).await;
    let second_url = serve(second.clone()).await;
    let third_url = serve(third.clone()).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.permanent_attempts = 1;
    options.tuning.retry.revive_after = Duration::from_secs(5);
    let harness = start_harness("hls-new-root-after-retirement", options);

    harness
        .handle
        .update_focus(focus(&[&first_url, &second_url]));
    wait_phase(&harness, SegmentedPhase::Failed).await;
    harness
        .handle
        .update_focus(focus(&[&first_url, &second_url, &third_url]));
    wait_phase(&harness, SegmentedPhase::Ready).await;

    assert_eq!(first.paths(), vec!["root"]);
    assert_eq!(second.paths(), vec!["root"]);
    assert_eq!(third.paths(), vec!["root", "child", "init", "segment"]);
    std::fs::remove_dir_all(&harness.root).ok();
}

fn focus(sources: &[&str]) -> ghostr_delivery::delivery_events::DeliveryFocus {
    let mut item = sized_item("stream", sources[0], 32, 4_000);
    item.meta.delivery = DeliveryKind::Hls;
    item.meta.urls = sources.iter().map(|source| (*source).to_owned()).collect();
    focus_now(vec![item], 0, 0)
}

async fn wait_phase(harness: &delivery_fixture::DeliveryHarness, expected: SegmentedPhase) {
    let changed = harness.segmented.notifier();
    tokio::time::timeout(Duration::from_secs(1), async {
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
    .expect("HLS phase transition before retired-root revival");
}
