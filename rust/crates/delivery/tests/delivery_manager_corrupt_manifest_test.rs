mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::wait_for_ranges;

#[tokio::test]
async fn corrupt_store_entry_does_not_block_another_posts_delivery() {
    let root = temp_directory("ghostr-manager-corrupt-entry");
    let corrupt_manifest = root.join("bad.ranges.json");
    std::fs::create_dir_all(&corrupt_manifest).expect("corrupt manifest");
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
    let root = harness.root.clone();
    // Store clear cannot remove the deliberately directory-shaped manifest.
    std::fs::remove_dir_all(corrupt_manifest).expect("remove corrupt manifest");
    harness.handle.clear().await.expect("clear delivery");
    drop(harness);
    std::fs::remove_dir_all(root).ok();
}
