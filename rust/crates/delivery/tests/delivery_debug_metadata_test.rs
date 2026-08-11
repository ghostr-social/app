mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;

#[tokio::test]
async fn delivery_focus_exposes_video_metadata_to_the_debug_gateway() {
    let origin = serve_recording("debug-metadata", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-debug-metadata", DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![sized_item("clip", &origin, 16, 90_000)],
        0,
        0,
    ));
    wait_for_ranges(&harness.store, "clip", &[(0, 1)]).await;

    let videos = harness.posts.videos();

    assert_eq!(videos.len(), 1);
    assert_eq!(videos[0].id, "clip");
    assert_eq!(videos[0].meta.duration_ms, Some(90_000));
    assert_eq!(videos[0].meta.urls, vec![origin]);
    assert!(!harness.root.join("clip.video").exists());
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
