mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use gateway_fixture::raw_http::unused_loopback_url;
use gateway_fixture::{media_client, native_download, video_id};
use ghostr_gateway::router::configured_router_with_client;
use ghostr_media_model::native_models::new_native_downloads;
use tower::ServiceExt;

#[tokio::test]
async fn returns_bad_gateway_when_the_video_origin_is_unavailable() {
    let downloads = new_native_downloads();
    downloads
        .lock()
        .await
        .insert(video_id(), native_download(&unused_loopback_url().await));
    let response = configured_router_with_client(downloads, media_client())
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
