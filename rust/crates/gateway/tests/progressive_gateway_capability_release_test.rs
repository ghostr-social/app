mod gateway_fixture;

use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::progressive_request::capability_request;
use tower::ServiceExt;

#[tokio::test]
async fn released_capability_fails_closed() {
    let harness = progressive_harness("ghostr-progressive-capability-release");
    harness.posts.insert("clip");
    let capability = harness.capabilities.issue("clip").await;
    assert!(harness.capabilities.release(capability.as_str()).await);
    let request = capability_request("clip", capability.as_str(), None);

    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    std::fs::remove_dir_all(harness.root).ok();
}
