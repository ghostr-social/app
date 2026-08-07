#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::engine::{DeliveryKind, VideoMeta};
use support::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn debug_gateway_does_not_expose_a_web_only_progressive_alias() {
    let harness = progressive_harness("debug-progressive-url-parity");
    harness.posts.insert_video(
        "clip",
        VideoMeta {
            urls: vec!["https://media.example/clip.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(1),
            duration_ms: Some(1),
        },
    );
    harness
        .store
        .set_total_len("clip", 1)
        .await
        .expect("total length");
    let request = Request::builder()
        .uri("/video?id=clip")
        .body(Body::empty())
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let _ = std::fs::remove_dir_all(harness.root);
}
