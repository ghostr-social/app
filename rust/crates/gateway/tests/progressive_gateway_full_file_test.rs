mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn serves_a_fully_present_video_with_complete_length_headers() {
    let harness = progressive_harness("ghostr-progressive-full");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(10))
        .await;
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

    let request = harness.video_request("clip", None).await;
    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "video/mp4");
    assert_eq!(response.headers()[CONTENT_LENGTH], "10");
    assert_eq!(response.headers()[ACCEPT_RANGES], "bytes");
    let body = to_bytes(response.into_body(), 64).await.expect("body");
    assert_eq!(&body[..], b"0123456789");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
