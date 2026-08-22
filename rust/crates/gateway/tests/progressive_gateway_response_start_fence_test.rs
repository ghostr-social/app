mod gateway_fixture;

use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;
use tokio_stream::StreamExt;
use tower::ServiceExt;

#[tokio::test]
async fn response_with_a_conflicting_length_cannot_read_new_single_response_bytes() {
    let harness = progressive_harness("ghostr-response-start-fence");
    harness.posts.insert("clip");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    let transfer = binding.transfer("https://cdn.example/video").unwrap();
    harness.store.bind_representation(binding).await.unwrap();
    harness
        .store
        .select_transfer(transfer.clone())
        .await
        .unwrap();
    harness.store.set_total_len("clip", 8).await.unwrap();

    let request = harness.video_request("clip", Some("bytes=0-7")).await;
    let response = harness.router.oneshot(request).await.unwrap();
    let mut body = response.into_body().into_data_stream();

    assert!(harness
        .store
        .begin_single_response(&transfer, 1, WholeBodyContract::Exact { expected_bytes: 9 },)
        .await
        .unwrap());
    harness
        .store
        .write_single_response_if_current(&transfer, 1, 0, b"newbytes!")
        .await
        .unwrap();

    let stopped = tokio::time::timeout(Duration::from_secs(1), body.next())
        .await
        .expect("new response generation terminates the old stream")
        .expect("body termination");
    assert!(stopped.is_err());
    std::fs::remove_dir_all(harness.root).expect("remove store");
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://cdn.example/video".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
