#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::debug::feed::{DebugFeedItem, DebugFeedStage};
use ghostr_engine::{DeliveryKind, VideoMeta};
use tower::ServiceExt;

#[tokio::test]
async fn debug_state_exposes_canonical_index_and_focus_distance() {
    let harness = progressive_harness("debug-focus-position");
    let items = [item("z-behind"), item("a-current"), item("m-next")];
    harness.posts.replace(
        items
            .iter()
            .map(|item| gateway_fixture::cache_video(item.id.clone(), item.meta.clone())),
    );
    harness
        .debug_feed
        .publish(1, DebugFeedStage::Settled, items.into());
    harness.debug_feed.select("a-current").expect("selection");

    let request = Request::get("/debug/api/state")
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state");
    let positions: Vec<_> = state["videos"]
        .as_array()
        .expect("videos")
        .iter()
        .map(|video| {
            (
                video["feed_index"].as_u64(),
                video["focus_distance"].as_i64(),
            )
        })
        .collect();

    assert_eq!(
        positions,
        [(Some(0), Some(-1)), (Some(1), Some(0)), (Some(2), Some(1))]
    );
}

fn item(id: &str) -> DebugFeedItem {
    DebugFeedItem {
        id: id.to_owned(),
        event_id: format!("event-{id}"),
        title: None,
        creator: "creator".to_owned(),
        created_at: 1,
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
    }
}
