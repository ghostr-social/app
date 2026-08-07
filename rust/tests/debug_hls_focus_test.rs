#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use rust_lib_ghostr::engine::{DeliveryKind, VideoMeta};
use rust_lib_ghostr::video::debug_feed::{DebugFeedItem, DebugFeedStage};
use support::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn hls_video_can_be_selected_through_the_debug_focus_api() {
    let harness = progressive_harness("debug-hls-focus");
    harness
        .debug_feed
        .publish(1, DebugFeedStage::Settled, vec![hls_item()]);
    let request = Request::builder()
        .method(Method::PUT)
        .uri("/debug/api/focus")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"id":"stream"}"#))
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        harness.debug_feed.snapshot().current_id.as_deref(),
        Some("stream")
    );
}

fn hls_item() -> DebugFeedItem {
    DebugFeedItem {
        id: "stream".to_owned(),
        event_id: "event".to_owned(),
        title: None,
        creator: "Ghost".to_owned(),
        created_at: 42,
        meta: VideoMeta {
            urls: vec!["https://media.example/live.m3u8".to_owned()],
            delivery: DeliveryKind::Hls,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    }
}
