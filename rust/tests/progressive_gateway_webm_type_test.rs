mod support;

use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use support::progressive::{progressive_harness, video_request};
use tower::ServiceExt;

const WEBM_HEADER: &[u8] = b"\x1a\x45\xdf\xa3\x9f\x42\x82\x84webm";

#[tokio::test]
async fn serves_cached_webm_bytes_with_their_browser_playback_type() {
    let harness = progressive_harness("ghostr-progressive-webm-type");
    harness.posts.insert("clip");
    harness
        .store
        .set_total_len("clip", WEBM_HEADER.len() as u64)
        .await
        .expect("total length");
    harness
        .store
        .write_range("clip", 0, WEBM_HEADER)
        .await
        .expect("bytes");

    let response = harness
        .router
        .oneshot(video_request("clip", None))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "video/webm");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
