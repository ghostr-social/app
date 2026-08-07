mod support;

use support::delivery::start_harness;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_options::DeliveryOptions;
use support::delivery_wait::wait_for_ranges;

#[tokio::test]
async fn delivery_focus_exposes_video_metadata_to_the_debug_gateway() {
    let origin = serve_recording("debug-metadata", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-debug-metadata", DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![sized_item("clip", &origin, 16, 90_000)],
        0,
        0,
    ));
    wait_for_ranges(&harness.store, "clip", &[(0, 16)]).await;

    let videos = harness.posts.videos();

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].id, "clip");
    assert_eq!(videos[0].meta.duration_ms, Some(90_000));
    assert_eq!(videos[0].meta.urls, vec![origin]);
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
