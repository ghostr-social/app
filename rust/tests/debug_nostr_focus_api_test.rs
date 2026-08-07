#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::discovery::event_cache::client_with_event_cache;
use rust_lib_ghostr::engine::{DeliveryKind, VideoMeta};
use rust_lib_ghostr::video::debug_feed::{DebugFeed, DebugFeedItem, DebugFeedStage};
use rust_lib_ghostr::video::delivery_events::{command_channel, DeliveryCommand};
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_progressive_debug;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::playback_demand::demand_channel;
use rust_lib_ghostr::video::progressive_route::ProgressiveState;
use std::sync::Arc;
use tower::ServiceExt;

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
            size_bytes: Some(10),
            duration_ms: Some(1_000),
        },
    }
}

#[tokio::test]
async fn browser_selection_updates_the_native_delivery_focus() {
    let harness = support::progressive::progressive_harness("debug-nostr-focus-api");
    let (delivery, mut commands) = command_channel();
    let feed = DebugFeed::new(delivery.clone(), vec!["wss://relay.example".to_owned()]);
    feed.publish(1, DebugFeedStage::Settled, vec![item("a"), item("b")]);
    commands.recv().await.expect("initial focus");
    let (demand, _) = demand_channel();
    let state = Arc::new(ProgressiveState {
        store: harness.store,
        demand,
        cache: harness.posts,
        network: harness.network,
        timing: Default::default(),
        debug_feed: feed,
    });
    let router = configured_router_with_progressive_debug(
        new_native_downloads(),
        HlsSessions::production(),
        support::fixtures::trusted_media_client(),
        state,
        delivery,
        Arc::new(client_with_event_cache()),
    );
    let request = Request::put("/debug/api/focus")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"id":"b"}"#))
        .expect("request");

    let response = router.oneshot(request).await.expect("response");
    let DeliveryCommand::Focus(focus) = commands.recv().await.expect("selected focus") else {
        panic!("expected focus");
    };

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(focus.current_index, 1);
}
