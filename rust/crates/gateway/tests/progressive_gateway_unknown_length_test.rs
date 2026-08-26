mod gateway_fixture;

use axum::http::header::RETRY_AFTER;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt as _;

#[tokio::test(start_paused = true)]
async fn answers_retry_later_while_the_total_length_is_still_unknown() {
    let harness = progressive_harness("ghostr-progressive-unsized");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", None)
        .await;

    let request = harness.video_request("clip", None).await;
    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[RETRY_AFTER], "1");
    let _ = std::fs::remove_dir_all(harness.root);
}
