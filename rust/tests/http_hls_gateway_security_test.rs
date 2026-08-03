mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_hls_client;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use support::fixtures::trusted_media_client;
use support::http::spawn_raw_server;
use tower::ServiceExt;

#[tokio::test]
async fn rejects_private_root_manifest_without_contacting_it() {
    let valid = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 8\r\n\r\n#EXTM3U\n";
    let (origin, upstream) = spawn_raw_server(valid).await;
    let sessions = HlsSessions::production();
    let id = sessions.acquire(vec![origin]).await.expect("session");
    let app = configured_router_with_hls_client(
        new_native_downloads(),
        sessions,
        MediaHttpClient::public().expect("public client"),
    );

    let response = app.oneshot(request(&id)).await.expect("gateway response");

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert!(!upstream.is_finished(), "private origin received a request");
    upstream.abort();
}

#[tokio::test]
async fn rejects_non_hls_mime_and_malformed_manifests() {
    let invalid_mime =
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 8\r\n\r\n#EXTM3U\n";
    let malformed = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 7\r\n\r\ninvalid";
    for upstream_response in [invalid_mime.as_slice(), malformed.as_slice()] {
        let (origin, upstream) = spawn_raw_server(upstream_response).await;
        let sessions = HlsSessions::production();
        let id = sessions.acquire(vec![origin]).await.expect("session");
        let app = configured_router_with_hls_client(
            new_native_downloads(),
            sessions,
            trusted_media_client(),
        );
        let response = app.oneshot(request(&id)).await.expect("gateway response");
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        upstream.await.expect("upstream request");
    }
}

fn request(id: &rust_lib_ghostr::video::hls_sessions::HlsSessionId) -> Request<Body> {
    Request::builder()
        .uri(format!("/hls/{}/index.m3u8", id.as_str()))
        .body(Body::empty())
        .expect("request")
}
