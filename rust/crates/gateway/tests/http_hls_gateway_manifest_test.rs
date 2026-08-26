mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use core::time::Duration;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::router_with_hls;
use gateway_fixture::raw_http::spawn_raw_server;
use ghostr_gateway::hls::sessions::{HlsSessionLimits, HlsSessions};
use tower::ServiceExt as _;

#[tokio::test]
async fn serves_a_root_manifest_with_only_gateway_resource_urls() {
    let response = b"HTTP/1.1 200 OK\r\n\
Content-Type: application/vnd.apple.mpegurl\r\n\
Content-Length: 72\r\nConnection: close\r\n\r\n\
#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000\nhttps://cdn.example/low.m3u8\n";
    let (origin, request) = spawn_raw_server(response).await;
    let sessions = sessions();
    let id = sessions.acquire(vec![origin]).await.expect("session");
    let router = router_with_hls(sessions, media_client());

    let response = router
        .oneshot(request_for(&format!("/hls/{}/index.m3u8", id.as_str())))
        .await
        .expect("gateway response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024).await.expect("body");
    let manifest = String::from_utf8(body.to_vec()).expect("manifest");
    assert!(!manifest.contains("https://cdn.example"));
    assert!(manifest.contains(&format!("/hls/{}/manifests/", id.as_str())));
    request.await.expect("origin request");
}

fn sessions() -> HlsSessions {
    let limits = HlsSessionLimits::new(2, Duration::from_secs(60), 8).expect("limits");
    HlsSessions::new(limits)
}

fn request_for(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .expect("request")
}
