mod gateway_fixture;

use axum::http::header::CONTENT_RANGE;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn rejects_a_range_that_starts_past_the_end_of_the_video() {
    let harness = progressive_harness("ghostr-progressive-unsat");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("total length");

    let request = harness.video_request("clip", Some("bytes=10-")).await;
    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(response.headers()[CONTENT_RANGE], "bytes */10");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
