mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::router_with_hls;
use gateway_fixture::raw_http::spawn_raw_server;
use ghostr_gateway::hls::sessions::HlsSessions;
use tower::ServiceExt;

#[tokio::test]
async fn valid_manifest_bytes_do_not_require_an_hls_mime() {
    assert_manifest_accepted(
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\n\r\n#EXTM3U\n",
    )
    .await;
    assert_manifest_accepted(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\n#EXTM3U\n").await;
}

async fn assert_manifest_accepted(response: &'static [u8]) {
    let (source, origin_request) = spawn_raw_server(response).await;
    let sessions = HlsSessions::production();
    let session = sessions.acquire(vec![source]).await.unwrap();
    let router = router_with_hls(sessions, media_client());
    let request = Request::builder()
        .uri(format!("/hls/{}/index.m3u8", session.as_str()))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    origin_request.await.unwrap();
}
