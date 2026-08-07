mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::StatusCode;
use gateway_fixture::progressive::{progressive_harness, video_request};
use tower::ServiceExt;

#[tokio::test(start_paused = true)]
async fn ends_a_stalled_stream_after_the_idle_timeout() {
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

    let response = harness
        .router
        .oneshot(video_request("clip", None))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 64).await.expect("body");
    assert_eq!(&body[..], b"01234");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
