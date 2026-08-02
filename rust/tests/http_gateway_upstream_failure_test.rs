mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::video::http_gateway::configured_router;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use support::fixtures::{native_download, video_id};
use support::http::unused_loopback_url;
use tower::ServiceExt;

#[tokio::test]
async fn returns_bad_gateway_when_the_video_origin_is_unavailable() {
    let downloads = new_native_downloads();
    downloads
        .lock()
        .await
        .insert(video_id(), native_download(&unused_loopback_url().await));
    let response = configured_router(downloads)
        .oneshot(
            Request::builder()
                .uri(format!("/video.mp4?id={}", video_id()))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}
