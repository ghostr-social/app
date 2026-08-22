mod gateway_fixture;

use bytes::Bytes;
use gateway_fixture::progressive::{progressive_harness, ProgressiveHarness};
use ghostr_delivery::playback_demand::{DemandLease, DemandState};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::{RepresentationBinding, SourceGeneration};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};
use tower::ServiceExt;

#[tokio::test]
async fn a_new_binding_generation_wakes_a_parked_response() {
    let mut harness = progressive_harness("ghostr-same-representation-rebind");
    harness.posts.insert("clip");
    let (mut catalog, first) = seed(&harness).await;
    let (mut body, lease) = parked_response(&mut harness).await;

    let second = catalog.upsert(PostId::new("clip"), meta(vec![source(), mirror()]));
    assert_eq!(first.representation(), second.representation());
    assert_ne!(first, second);
    harness.store.bind_representation(second).await.unwrap();

    assert!(next_within(&mut body).await.is_err());
    assert_eq!(
        demand_within(&mut harness).await,
        DemandState::Released(lease.consumer())
    );
    std::fs::remove_dir_all(harness.root).unwrap();
}

async fn seed(harness: &ProgressiveHarness) -> (Catalog, RepresentationBinding) {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta(vec![source()]));
    let transfer = binding.transfer(&source()).unwrap();
    harness
        .store
        .bind_representation(binding.clone())
        .await
        .unwrap();
    harness
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    harness
        .store
        .accept_generation(&transfer, generation())
        .await
        .unwrap();
    harness
        .store
        .write_range("clip", 0, b"01234")
        .await
        .unwrap();
    (catalog, binding)
}

async fn parked_response(
    harness: &mut ProgressiveHarness,
) -> (
    impl Stream<Item = Result<Bytes, axum::Error>> + Unpin,
    DemandLease,
) {
    let request = harness.video_request("clip", Some("bytes=0-9")).await;
    let response = harness.router.clone().oneshot(request).await.unwrap();
    let mut body = response.into_body().into_data_stream();
    assert_eq!(&next_within(&mut body).await.unwrap()[..], b"01234");
    let DemandState::Blocked(lease) = demand_within(harness).await else {
        panic!("missing bytes must block");
    };
    (body, lease)
}

async fn next_within(
    body: &mut (impl Stream<Item = Result<Bytes, axum::Error>> + Unpin),
) -> Result<Bytes, axum::Error> {
    tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("response body timeout")
        .expect("response body termination")
}

async fn demand_within(harness: &mut ProgressiveHarness) -> DemandState {
    tokio::time::timeout(Duration::from_secs(1), harness.demand.recv())
        .await
        .expect("demand timeout")
        .expect("demand channel")
}

fn generation() -> SourceGeneration {
    SourceGeneration::try_new(source(), "\"one\"", 10).unwrap()
}

fn meta(urls: Vec<String>) -> VideoMeta {
    VideoMeta {
        urls,
        delivery: DeliveryKind::Progressive,
        sha256: Some("same-content".to_owned()),
        size_bytes: Some(10),
        duration_ms: Some(1_000),
    }
}

fn source() -> String {
    "https://one.example/video.mp4".to_owned()
}

fn mirror() -> String {
    "https://two.example/video.mp4".to_owned()
}
