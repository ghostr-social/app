#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use rust_lib_ghostr::engine::{DeliveryKind, VideoMeta};
use rust_lib_ghostr::video::debug_feed::{DebugFeedItem, DebugFeedStage};
use support::progressive::progressive_harness;
use tower::ServiceExt;

fn item() -> DebugFeedItem {
    DebugFeedItem {
        id: "clip".to_owned(),
        event_id: "nostr-event".to_owned(),
        title: Some("Relay video".to_owned()),
        creator: "Nostr creator".to_owned(),
        created_at: 42,
        meta: VideoMeta {
            urls: vec!["https://media.example/clip.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(100),
            duration_ms: Some(1_000),
        },
    }
}

#[tokio::test]
async fn debug_state_exposes_nostr_feed_and_post_metadata() {
    let harness = progressive_harness("debug-nostr-state");
    let video = item();
    harness
        .posts
        .insert_video(video.id.clone(), video.meta.clone());
    harness
        .debug_feed
        .publish(4, DebugFeedStage::Settled, vec![video]);
    let request = Request::get("/debug/api/state")
        .body(Body::empty())
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state");

    assert_eq!(state["nostr"]["stage"], "settled");
    assert_eq!(state["nostr"]["discovered_count"], 1);
    assert_eq!(state["videos"][0]["nostr_event_id"], "nostr-event");
    assert_eq!(state["videos"][0]["title"], "Relay video");
    assert_eq!(state["videos"][0]["creator"], "Nostr creator");
}
