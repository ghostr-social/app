mod gateway_fixture;

use axum::http::header::RETRY_AFTER;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt as _;

#[tokio::test]
async fn known_length_without_a_binding_is_retryable() {
    let harness = progressive_harness("ghostr-progressive-initial-unbound");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", None)
        .await;
    let request = harness.video_request("clip", None).await;
    harness.store.clear().await.expect("valid test fixture");
    harness
        .store
        .set_total_len("clip", 8)
        .await
        .expect("valid test fixture");

    let response = harness
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("valid test fixture");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[RETRY_AFTER], "1");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
