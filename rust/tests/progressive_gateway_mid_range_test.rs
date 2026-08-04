mod support;

use axum::body::to_bytes;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::StatusCode;
use support::progressive::{progressive_harness, video_request};
use tower::ServiceExt;

#[tokio::test]
async fn serves_a_mid_file_range_with_a_correct_content_range() {
    let harness = progressive_harness("ghostr-progressive-mid");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("total length");
    harness
        .store
        .write_range("clip", 0, b"0123456789")
        .await
        .expect("bytes");

    let response = harness
        .router
        .oneshot(video_request("clip", Some("bytes=2-5")))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()[CONTENT_TYPE], "video/mp4");
    assert_eq!(response.headers()[CONTENT_LENGTH], "4");
    assert_eq!(response.headers()[CONTENT_RANGE], "bytes 2-5/10");
    assert_eq!(response.headers()[ACCEPT_RANGES], "bytes");
    let body = to_bytes(response.into_body(), 64).await.expect("body");
    assert_eq!(&body[..], b"2345");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
