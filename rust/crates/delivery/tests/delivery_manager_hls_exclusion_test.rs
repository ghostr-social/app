mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::DeliveryKind;

#[tokio::test]
async fn delivery_manager_excludes_hls_items_from_progressive_storage() {
    let hls_origin = serve_recording("hls", media_body(), hit_log()).await;
    let progressive_origin = serve_recording("progressive", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-hls-exclusion", DeliveryOptions::default());
    let mut hls = sized_item("hls", &hls_origin, 16, 1_000);
    hls.meta.delivery = DeliveryKind::Hls;

    harness.handle.update_focus(focus_now(
        vec![
            hls,
            sized_item("progressive", &progressive_origin, 16, 1_000),
        ],
        1,
        0,
    ));

    wait_for_ranges(&harness.store, "progressive", &[(0, 16)]).await;
    assert!(!harness.posts.contains("hls"));
    assert!(harness.posts.contains("progressive"));
    assert!(harness
        .store
        .present_ranges("hls")
        .await
        .expect("HLS ranges")
        .is_empty());
    std::fs::remove_dir_all(&harness.root).ok();
}
