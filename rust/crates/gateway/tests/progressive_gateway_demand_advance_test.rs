mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use tower::ServiceExt as _;

#[tokio::test]
async fn waiting_response_advances_its_existing_consumer_lease() {
    let mut harness = progressive_harness("ghostr-progressive-demand-advance");
    let total = PLAYBACK_SLICE_BYTES * 2 + 1;
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(total))
        .await;
    harness
        .store
        .set_total_len("clip", total)
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_range("clip", 0, &[7])
        .await
        .expect("valid test fixture");
    let request = harness.video_request("clip", Some("bytes=0-")).await;
    let response = harness
        .router
        .oneshot(request)
        .await
        .expect("valid test fixture");
    let DemandState::Blocked(first) = harness.demand.recv().await.expect("valid test fixture")
    else {
        panic!("first state must block");
    };
    harness
        .store
        .write_range(
            "clip",
            first.range().start,
            &vec![7; first.range().len() as usize],
        )
        .await
        .expect("valid test fixture");

    let DemandState::Advanced(next) = harness.demand.recv().await.expect("valid test fixture")
    else {
        panic!("second state must advance");
    };

    assert_eq!(next.consumer(), first.consumer());
    assert_eq!(next.range().start, first.range().end);
    drop(response);
    std::fs::remove_dir_all(harness.root).ok();
}
