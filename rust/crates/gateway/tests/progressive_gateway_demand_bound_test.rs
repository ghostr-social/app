use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;

mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

const LARGE_VIDEO_BYTES: u64 = 8 * 1024 * 1024;

#[tokio::test]
async fn an_open_ended_player_request_demands_only_the_next_bounded_window() {
    let mut harness = progressive_harness("ghostr-progressive-demand-bound");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", LARGE_VIDEO_BYTES)
        .await
        .unwrap();
    harness.store.write_range("clip", 0, &[7]).await.unwrap();

    let request = harness.video_request("clip", Some("bytes=0-")).await;
    let response = harness.router.oneshot(request).await.unwrap();
    let signal = harness.demand.recv().await.expect("demand signal");

    assert_eq!(signal.range.start, 1);
    assert_eq!(signal.range.end - signal.range.start, PLAYBACK_SLICE_BYTES);
    drop(response);
    std::fs::remove_dir_all(harness.root).ok();
}
