mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::header::{CONTENT_LENGTH, RANGE};
use axum::http::{HeaderValue, StatusCode};
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt as _;

#[tokio::test]
async fn duplicate_progressive_ranges_are_ignored_as_one_ambiguous_set() {
    let harness = progressive_harness("ghostr-progressive-duplicate-range");
    harness.posts.insert("clip");
    harness
        .bind_video("clip", "https://cdn.example/clip.mp4", Some(10))
        .await;
    harness
        .store
        .set_total_len("clip", 10)
        .await
        .expect("valid test fixture");
    harness
        .store
        .write_range("clip", 0, b"0123456789")
        .await
        .expect("valid test fixture");

    let mut request = harness.video_request("clip", Some("bytes=2-5")).await;
    request
        .headers_mut()
        .append(RANGE, HeaderValue::from_static("bytes=6-7"));
    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_LENGTH], "10");
    assert_eq!(
        to_bytes(response.into_body(), 16)
            .await
            .expect("valid test fixture"),
        "0123456789"
    );
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
