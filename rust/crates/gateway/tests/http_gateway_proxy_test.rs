mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{Request, StatusCode};
use gateway_fixture::raw_http::spawn_raw_server;
use gateway_fixture::{media_client, native_download, video_id};
use ghostr_gateway::http_gateway::configured_router_with_client;
use ghostr_media_model::native_models::new_native_downloads;
use tower::ServiceExt;

#[tokio::test]
async fn proxies_video_ranges_and_streaming_headers_from_the_origin() {
    let (url, upstream_request) = spawn_raw_server(
        b"HTTP/1.1 206 Partial Content\r\nContent-Type: video/mp4\r\nContent-Length: 5\r\nContent-Range: bytes 0-4/5\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\nvideo",
    )
    .await;
    let downloads = new_native_downloads();
    downloads
        .lock()
        .await
        .insert(video_id(), native_download(&url));
    let response = configured_router_with_client(downloads, media_client())
        .oneshot(
            Request::builder()
                .uri(format!("/video.mp4?id={}", video_id()))
                .header(RANGE, "bytes=0-4")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()[CONTENT_TYPE], "video/mp4");
    assert_eq!(response.headers()[CONTENT_LENGTH], "5");
    assert_eq!(response.headers()[CONTENT_RANGE], "bytes 0-4/5");
    assert_eq!(response.headers()[ACCEPT_RANGES], "bytes");
    let body = to_bytes(response.into_body(), 5).await.expect("body");
    assert_eq!(&body[..], b"video");
    let request = upstream_request.await.expect("upstream request");
    let request = String::from_utf8_lossy(&request).to_ascii_lowercase();
    assert!(request.contains("range: bytes=0-4"));
}
