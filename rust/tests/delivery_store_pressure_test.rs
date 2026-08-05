//! A full device must not cost the user a video. Bytes that could not
//! land say nothing about the host that sent them, so an out-of-space
//! write must not spend the source's attempt budget: device pass 3
//! recorded sixteen refusals in one second, a `Video unavailable` panel
//! and seven failed player initializations.

mod store_space;
mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::Duration;
use store_space::{discard, limits, spaced_store};
use support::delivery::start_harness_with_store;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_options::DeliveryOptions;
use tokio::time::Instant;

#[tokio::test]
async fn delivery_keeps_a_post_playable_while_the_store_is_full() {
    let fixture = spaced_store("ghostr-delivery-full", limits(u64::MAX, 1_000), 1_000);
    std::fs::create_dir_all(&fixture.root).expect("store root");
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let root = fixture.root.clone();
    let harness =
        start_harness_with_store(Arc::new(fixture.store), root, DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &origin, 16, 1_000)],
        0,
        5_000,
    ));

    wait_for_refusals(&harness.store, 6).await;

    assert!(
        harness.posts.contains("aa11"),
        "the store being full must not retire the post's only source"
    );
    discard(&fixture.root);
}

/// More refusals than the retry policy's transient attempt budget, so a
/// source that was being charged for them would already be retired.
async fn wait_for_refusals(store: &PartialRangeStore, wanted: u64) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while store.refusals() < wanted {
        assert!(
            Instant::now() < deadline,
            "timed out at {} refusals",
            store.refusals()
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
