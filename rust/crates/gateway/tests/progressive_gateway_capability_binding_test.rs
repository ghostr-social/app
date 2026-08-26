mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::progressive_request::capability_request;
use tower::ServiceExt as _;

#[tokio::test]
async fn capability_for_another_post_fails_closed() {
    let harness = progressive_harness("ghostr-progressive-capability-binding");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", 1)
        .await
        .expect("total length");
    harness
        .bind_video("other", "https://cdn.example/other.mp4", Some(1))
        .await;
    let capability = harness.issue_video_asset("other").await;
    let request = capability_request("clip", capability.as_str(), None);

    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
