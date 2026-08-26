mod gateway_fixture;

use core::time::Duration;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::playback_demand::DemandState;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use tokio_stream::StreamExt as _;
use tower::ServiceExt as _;

#[tokio::test]
async fn an_open_response_fails_instead_of_mixing_a_new_representation() {
    let mut harness = progressive_harness("ghostr-response-representation-fence");
    harness.posts.insert("clip");
    let mut catalog = Catalog::new();
    let first = catalog.upsert(PostId::new("clip"), meta("old", 10));
    harness
        .store
        .bind_representation(first)
        .await
        .expect("valid test fixture");
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
    let mut body = response.into_body().into_data_stream();
    let first = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("first body chunk")
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert_eq!(&first[..], b"01234");
    let demand = tokio::time::timeout(Duration::from_secs(1), harness.demand.recv())
        .await
        .expect("demand timeout")
        .expect("old representation demand");
    let DemandState::Blocked(lease) = demand else {
        panic!("first demand state must block");
    };

    let second = catalog.upsert(PostId::new("clip"), meta("new", 10));
    harness
        .store
        .bind_representation(second)
        .await
        .expect("valid test fixture");
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_range("clip", 0, b"abcdefghij")
        .await
        .expect("valid test fixture");

    let stopped = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("representation switch must terminate promptly")
        .expect("body termination");
    assert!(stopped.is_err());
    let released = harness.demand.recv().await.expect("released stale demand");
    assert_eq!(released, DemandState::Released(lease.consumer()));
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

fn meta(name: &str, size: u64) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{name}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(format!("{name}-digest")),
        size_bytes: Some(size),
        duration_ms: Some(1_000),
    }
}
