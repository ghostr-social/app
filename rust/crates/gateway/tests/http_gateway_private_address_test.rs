mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use gateway_fixture::raw_http::spawn_raw_server;
use gateway_fixture::{native_download, video_id};
use ghostr_gateway::router::configured_router;
use ghostr_media_model::native_models::new_native_downloads;
use tower::ServiceExt;

#[tokio::test]
async fn does_not_proxy_a_private_network_media_url() {
    let (url, upstream) = spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
    let downloads = new_native_downloads();
    downloads
        .lock()
        .await
        .insert(video_id(), native_download(&url));

    let response = configured_router(downloads)
        .expect("router")
        .oneshot(
            Request::builder()
                .uri(format!("/video.mp4?id={}", video_id()))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert!(
        !upstream.is_finished(),
        "private endpoint received a request"
    );
    upstream.abort();
}
