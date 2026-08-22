mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn rejects_a_post_the_manager_never_registered() {
    let harness = progressive_harness("ghostr-progressive-unknown");
    harness.posts.insert("other");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", None)
        .await;

    let request = harness.video_request("clip", None).await;
    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let _ = std::fs::remove_dir_all(harness.root);
}
