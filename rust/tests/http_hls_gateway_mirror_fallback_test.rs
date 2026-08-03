mod support;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_hls_client;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use support::fixtures::trusted_media_client;
use support::http::spawn_raw_server;
use tower::ServiceExt;

#[tokio::test]
async fn falls_back_to_the_next_manifest_mirror() {
    let invalid =
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 7\r\n\r\ninvalid";
    let valid = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 8\r\n\r\n#EXTM3U\n";
    let (first, first_request) = spawn_raw_server(invalid).await;
    let (second, second_request) = spawn_raw_server(valid).await;
    let sessions = HlsSessions::production();
    let id = sessions
        .acquire(vec![first, second])
        .await
        .expect("session");
    let app =
        configured_router_with_hls_client(new_native_downloads(), sessions, trusted_media_client());
    let request = Request::builder()
        .uri(format!("/hls/{}/index.m3u8", id.as_str()))
        .body(Body::empty())
        .expect("request");

    let response = app.oneshot(request).await.expect("gateway response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 64).await.expect("body");
    assert_eq!(&body[..], b"#EXTM3U\n");
    first_request.await.expect("first mirror request");
    second_request.await.expect("second mirror request");
}
