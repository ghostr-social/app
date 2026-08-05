mod support;

use support::delivery::start_harness_at;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_options::DeliveryOptions;
use support::delivery_wait::wait_for_ranges;
use support::fixtures::temp_directory;

#[tokio::test]
async fn corrupt_store_entry_does_not_block_another_posts_delivery() {
    let root = temp_directory("ghostr-manager-corrupt-entry");
    std::fs::create_dir_all(root.join("bad.ranges.json")).expect("corrupt manifest");
    let broken_origin = serve_recording("broken", media_body(), hit_log()).await;
    let live_origin = serve_recording("live", media_body(), hit_log()).await;
    let harness = start_harness_at(root, DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![
            sized_item("bad", &broken_origin, 16, 1_000),
            sized_item("good", &live_origin, 16, 1_000),
        ],
        1,
        0,
    ));

    wait_for_ranges(&harness.store, "good", &[(0, 16)]).await;
    assert!(harness.posts.contains("good"));
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
