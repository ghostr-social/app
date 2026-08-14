mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use ghostr_engine::playback::PLAYBACK_SLICE_BYTES;
use tower::ServiceExt;

#[tokio::test]
async fn waiting_response_advances_its_existing_consumer_lease() {
    let mut harness = progressive_harness("ghostr-progressive-demand-advance");
    let total = PLAYBACK_SLICE_BYTES * 2 + 1;
    harness.posts.insert("clip");
    harness.store.set_total_len("clip", total).await.unwrap();
    harness.store.write_range("clip", 0, &[7]).await.unwrap();
    let request = harness.video_request("clip", Some("bytes=0-")).await;
    let response = harness.router.oneshot(request).await.unwrap();
    let DemandState::Blocked(first) = harness.demand.recv().await.unwrap() else {
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
        .unwrap();

    let DemandState::Advanced(next) = harness.demand.recv().await.unwrap() else {
        panic!("second state must advance");
    };

    assert_eq!(next.consumer(), first.consumer());
    assert_eq!(next.range().start, first.range().end);
    drop(response);
    std::fs::remove_dir_all(harness.root).ok();
}
