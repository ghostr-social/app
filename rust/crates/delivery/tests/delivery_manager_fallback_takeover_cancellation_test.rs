mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::ControlledOrigin;
use delivery_fixture::items::focus_now;
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::{DeliveryKind, EngineParams, PostId, VideoMeta};

const TOTAL: u64 = 32;

#[tokio::test]
async fn exact_fallback_takeover_cancels_a_held_primary_before_late_bytes() {
    let mut origin = ControlledOrigin::serve(TOTAL).await;
    let primary = origin.url_for("primary");
    let fallback = origin.url_for("fallback");
    let item = mirrored_item(&primary, &fallback);
    let harness = start_harness("fallback-takeover-cancel", serial_options());
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    let held = tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("primary request starts");
    assert_eq!(held.path, "/primary.mp4");

    publish_fallback(&harness).await;
    harness.handle.storage_changed();

    let cancelled = tokio::time::timeout(Duration::from_secs(30), async {
        while held.is_open() {
            tokio::task::yield_now().await;
        }
    })
    .await;
    assert!(
        cancelled.is_ok(),
        "primary remained open: {:?}",
        harness.handle.plan_history().last()
    );
    assert!(!held.send_byte().await, "late primary bytes stay fenced");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(harness.root).ok();
}

async fn publish_fallback(harness: &delivery_fixture::DeliveryHarness) {
    harness
        .store
        .write_range("post", 0, &[7; TOTAL as usize])
        .await
        .expect("publish canonical fallback bytes");
}

fn mirrored_item(primary: &str, fallback: &str) -> FocusItem {
    FocusItem {
        post: PostId::new("post"),
        meta: VideoMeta {
            urls: vec![primary.to_owned(), fallback.to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(TOTAL),
            duration_ms: Some(1_000),
        },
    }
}

fn serial_options() -> DeliveryOptions {
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: TOTAL,
        balanced_concurrency: 1,
        ..options.params
    };
    options
}
