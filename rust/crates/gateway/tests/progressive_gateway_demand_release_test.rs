mod gateway_fixture;

use axum::body::to_bytes;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use tower::ServiceExt as _;

#[tokio::test]
async fn completed_response_releases_its_demand_lease() {
    let mut harness = progressive_harness("ghostr-progressive-demand-release");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(10))
        .await;
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_range("clip", 0, b"01234")
        .await
        .expect("valid test fixture");
    let request = harness.video_request("clip", Some("bytes=0-9")).await;
    let response = harness
        .router
        .oneshot(request)
        .await
        .expect("valid test fixture");
    let DemandState::Blocked(lease) = harness.demand.recv().await.expect("valid test fixture")
    else {
        panic!("missing blocked state");
    };
    harness
        .store
        .write_range("clip", 5, b"56789")
        .await
        .expect("valid test fixture");

    let body = to_bytes(response.into_body(), 64)
        .await
        .expect("valid test fixture");
    let released = harness.demand.recv().await.expect("valid test fixture");

    assert_eq!(&body[..], b"0123456789");
    assert_eq!(released, DemandState::Released(lease.consumer()));
    std::fs::remove_dir_all(harness.root).ok();
}
