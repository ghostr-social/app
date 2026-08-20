mod gateway_fixture;

use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

const WEBM_HEADER: &[u8] = b"\x1a\x45\xdf\xa3\x9f\x42\x82\x84webm";

#[tokio::test]
async fn does_not_claim_unsupported_webm_playback() {
    let harness = progressive_harness("ghostr-progressive-webm-type");
    harness.posts.insert("clip");
    harness
        .bind_video(
            "clip",
            "https://cdn.example/clip.webm",
            Some(WEBM_HEADER.len() as u64),
        )
        .await;
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

    let request = harness.video_request("clip", None).await;
    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "video/mp4");
    std::fs::remove_dir_all(harness.root).expect("remove store");
}
