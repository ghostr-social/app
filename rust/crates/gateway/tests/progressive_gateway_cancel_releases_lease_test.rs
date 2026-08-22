mod gateway_fixture;

use gateway_fixture::free_space::{discard, limits, spaced_store};
use gateway_fixture::progressive::progressive_harness_with_store;
use ghostr_delivery::playback_demand::DemandState;
use ghostr_gateway::progressive::route::ProgressiveTiming;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use tower::ServiceExt;

#[tokio::test]
async fn canceled_waiting_response_releases_its_store_lease_immediately() {
    let fixture = spaced_store("ghostr-cancel-lease", limits(1_000, 1_000), 2_000);
    let root = fixture.root.clone();
    let space = fixture.space.clone();
    let store = Arc::new(fixture.store);
    let mut harness =
        progressive_harness_with_store(root.clone(), store.clone(), ProgressiveTiming::default());
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(800))
        .await;
    store.set_total_len("clip", 800).await.expect("total");
    store
        .write_range("clip", 0, &[7; 400])
        .await
        .expect("prefix");
    let request = harness.video_request("clip", Some("bytes=0-799")).await;
    let response = harness.router.oneshot(request).await.expect("response");
    let mut body = response.into_body().into_data_stream();
    assert_eq!(body.next().await.unwrap().unwrap().len(), 400);
    let DemandState::Blocked(lease) = harness.demand.recv().await.expect("missing-byte demand")
    else {
        panic!("first demand state must block");
    };
    let mut capacity = store.capacity_changes();

    drop(body);
    let released = tokio::time::timeout(Duration::from_millis(100), harness.demand.recv())
        .await
        .expect("cancellation releases demand without idle timeout")
        .expect("released demand state");
    assert_eq!(released, DemandState::Released(lease.consumer()));
    tokio::time::timeout(Duration::from_millis(100), capacity.changed())
        .await
        .expect("cancellation releases lease without idle timeout")
        .expect("capacity channel");
    space.set(0);

    assert_eq!(store.enforce_capacity().await, 400);
    discard(&root);
}
