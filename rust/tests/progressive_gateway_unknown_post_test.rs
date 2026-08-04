mod support;

use axum::http::StatusCode;
use support::progressive::{progressive_harness, video_request};
use tower::ServiceExt;

#[tokio::test]
async fn rejects_a_post_the_manager_never_registered() {
    let harness = progressive_harness("ghostr-progressive-unknown");
    harness.posts.insert("other");

    let response = harness
        .router
        .oneshot(video_request("clip", None))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let _ = std::fs::remove_dir_all(harness.root);
}
