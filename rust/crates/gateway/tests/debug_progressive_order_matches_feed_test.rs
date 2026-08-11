#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::debug::feed::{DebugFeedItem, DebugFeedStage};
use ghostr_engine::{DeliveryKind, VideoMeta};
use tower::ServiceExt;

#[tokio::test]
async fn debug_state_preserves_non_lexical_feed_order() {
    let harness = progressive_harness("debug-feed-order");
    let items = [item("z-current", 3), item("a-next", 2), item("m-later", 1)];
    harness.posts.replace(
        items
            .iter()
            .map(|item| gateway_fixture::cache_video(item.id.clone(), item.meta.clone())),
    );
    harness
        .debug_feed
        .publish(1, DebugFeedStage::Settled, items.into());

    let request = Request::get("/debug/api/state")
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state");
    let ids: Vec<_> = state["videos"]
        .as_array()
        .expect("videos")
        .iter()
        .map(|video| video["id"].as_str().expect("id"))
        .collect();

    assert_eq!(ids, ["z-current", "a-next", "m-later"]);
}

fn item(id: &str, created_at: u64) -> DebugFeedItem {
    DebugFeedItem {
        id: id.to_owned(),
        event_id: format!("event-{id}"),
        title: None,
        creator: "creator".to_owned(),
        created_at,
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
    }
}
