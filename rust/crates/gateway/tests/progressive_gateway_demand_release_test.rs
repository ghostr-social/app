mod gateway_fixture;

use axum::body::to_bytes;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use tower::ServiceExt;

#[tokio::test]
async fn completed_response_releases_its_demand_lease() {
    let mut harness = progressive_harness("ghostr-progressive-demand-release");
    harness.posts.insert("clip");
    harness.store.set_total_len("clip", 10).await.unwrap();
    harness
        .store
        .write_range("clip", 0, b"01234")
        .await
        .unwrap();
    let request = harness.video_request("clip", Some("bytes=0-9")).await;
    let response = harness.router.oneshot(request).await.unwrap();
    let DemandState::Blocked(lease) = harness.demand.recv().await.unwrap() else {
        panic!("missing blocked state");
    };
    harness
        .store
        .write_range("clip", 5, b"56789")
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), 64).await.unwrap();
    let released = harness.demand.recv().await.unwrap();

    assert_eq!(&body[..], b"0123456789");
    assert_eq!(released, DemandState::Released(lease.consumer()));
    std::fs::remove_dir_all(harness.root).ok();
}
