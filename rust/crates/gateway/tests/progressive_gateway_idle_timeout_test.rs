mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::header::{CONTENT_LENGTH, CONTENT_RANGE};
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn fails_a_promised_range_after_the_idle_timeout() {
    let harness = progressive_harness("ghostr-progressive-idle");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("total length");
    harness
        .store
        .write_range("clip", 0, b"01234")
        .await
        .expect("head bytes");

    let request = harness.video_request("clip", Some("bytes=0-9")).await;
    let response = harness.router.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()[CONTENT_LENGTH], "10");
    assert_eq!(response.headers()[CONTENT_RANGE], "bytes 0-9/10");

    let body = to_bytes(response.into_body(), 64).await;
    assert!(body.is_err(), "a promised range must not truncate cleanly");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
