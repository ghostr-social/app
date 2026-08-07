#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::raw_http::spawn_raw_server;
use ghostr_delivery::debug::feed::{DebugFeedItem, DebugFeedStage};
use ghostr_engine::{DeliveryKind, VideoMeta};
use serde_json::Value;
use tower::ServiceExt;

const MANIFEST: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 8\r\nConnection: close\r\n\r\n#EXTM3U\n";

#[tokio::test]
async fn web_hls_uses_the_shared_gateway_session_and_never_enters_cache() {
    let (source, upstream) = spawn_raw_server(MANIFEST).await;
    let harness = progressive_harness("debug-hls-playback");
    harness
        .debug_feed
        .publish(1, DebugFeedStage::Settled, vec![hls_item(source)]);

    let state = json(
        &harness.router,
        request(Method::GET, "/debug/api/state", ""),
    )
    .await;
    assert_eq!(state["hls_videos"][0]["id"], "stream");
    assert!(!harness.posts.contains("stream"));

    let session = json(
        &harness.router,
        request(Method::POST, "/debug/api/hls", r#"{"id":"stream"}"#),
    )
    .await;
    let playback = session["playback_url"].as_str().expect("playback URL");
    let response = harness
        .router
        .clone()
        .oneshot(request(Method::GET, playback, ""))
        .await
        .expect("manifest");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(to_bytes(response.into_body(), 32)
        .await
        .expect("body")
        .starts_with(b"#EXTM3U"));
    upstream.await.expect("upstream request");

    let id = session["session_id"].as_str().expect("session id");
    let response = harness
        .router
        .clone()
        .oneshot(request(Method::DELETE, &format!("/debug/api/hls/{id}"), ""))
        .await
        .expect("release");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(harness
        .hls_sessions
        .sources(&ghostr_gateway::hls::sessions::HlsSessionId::parse(id).expect("valid id"))
        .await
        .is_none());
}

fn hls_item(source: String) -> DebugFeedItem {
    DebugFeedItem {
        id: "stream".to_owned(),
        event_id: "event".to_owned(),
        title: Some("Live stream".to_owned()),
        creator: "Ghost".to_owned(),
        created_at: 42,
        meta: VideoMeta {
            urls: vec![source],
            delivery: DeliveryKind::Hls,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}

async fn json(router: &axum::Router, request: Request<Body>) -> Value {
    let response = router.clone().oneshot(request).await.expect("response");
    assert!(response.status().is_success(), "{}", response.status());
    let body = to_bytes(response.into_body(), 16_384).await.expect("body");
    serde_json::from_slice(&body).expect("JSON")
}

fn request(method: Method, uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_owned()))
        .expect("request")
}
