mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use tower::ServiceExt;

#[tokio::test]
async fn retries_when_the_representation_disappears_while_length_is_learned() {
    let harness = progressive_harness("ghostr-snapshot-binding-disappears");
    harness.posts.insert("clip");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), meta());
    harness.store.bind_representation(binding).await.unwrap();
    let request = harness.video_request("clip", None).await;
    let response = tokio::spawn(harness.router.clone().oneshot(request));

    tokio::task::yield_now().await;
    harness.store.present_ranges("clip").await.unwrap();
    harness.store.clear().await.unwrap();
    harness.store.set_total_len("clip", 8).await.unwrap();

    assert_eq!(
        response.await.unwrap().unwrap().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    std::fs::remove_dir_all(harness.root).unwrap();
}

fn meta() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://video.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: Some("digest".to_owned()),
        size_bytes: Some(8),
        duration_ms: Some(1_000),
    }
}
