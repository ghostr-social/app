mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::media_client;
use gateway_fixture::progressive_hls::router_with_hls;
use gateway_fixture::raw_http::spawn_response_sequence;
use ghostr_gateway::hls::sessions::HlsSessions;
use tower::ServiceExt;

#[tokio::test]
async fn proxies_nested_manifests_keys_maps_and_segments() {
    let root = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 51\r\nConnection: close\r\n\r\n#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nvariant.m3u8\n";
    let nested = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 108\r\nConnection: close\r\n\r\n#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nseg.m4s\n#EXT-X-ENDLIST\n";
    let asset = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 4\r\nConnection: close\r\n\r\nkey!";
    let (origin, requests) = spawn_response_sequence(vec![root, nested, asset]).await;
    let sessions = HlsSessions::production();
    let id = sessions.acquire(vec![origin]).await.expect("session");
    let app = router_with_hls(sessions, media_client());

    let root_body = response_text(&app, &format!("/hls/{}/index.m3u8", id.as_str())).await;
    let nested_path = root_body
        .lines()
        .find(|line| line.starts_with('/'))
        .expect("nested URI");
    let nested_body = response_text(&app, nested_path).await;
    assert!(!nested_body.contains("key.bin"));
    assert!(!nested_body.contains("init.mp4"));
    assert!(!nested_body.contains("seg.m4s"));
    let key_path = nested_body.split("URI=\"").nth(1).expect("key URI");
    let key_path = key_path.split('"').next().expect("key path");
    let response = app
        .clone()
        .oneshot(request(key_path))
        .await
        .expect("asset response");
    let body = to_bytes(response.into_body(), 16)
        .await
        .expect("asset body");
    assert_eq!(&body[..], b"key!");
    requests.await.expect("upstream requests");
}

async fn response_text(app: &axum::Router, path: &str) -> String {
    let response = app
        .clone()
        .oneshot(request(path))
        .await
        .expect("gateway response");
    let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("body");
    String::from_utf8(body.to_vec()).expect("UTF-8 manifest")
}

fn request(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("request")
}
