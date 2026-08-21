#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use gateway_fixture::commands::next_control;
use ghostr_delivery::debug::feed::{DebugFeed, DebugFeedItem, DebugFeedStage};
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_engine::{DeliveryKind, VideoMeta};
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::route::ProgressiveState;
use ghostr_gateway::router::{configured_router_with_progressive_debug, GatewayRouterResources};
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
    let harness = gateway_fixture::progressive::progressive_harness("debug-nostr-focus-api");
    let (delivery, mut commands) = command_channel();
    let feed = DebugFeed::new(delivery.clone(), vec!["wss://relay.example".to_owned()]);
    feed.publish(1, DebugFeedStage::Settled, vec![item("a"), item("b")]);
    next_control(&mut commands).await;
    let (demand, _) = demand_channel();
    let state = Arc::new(ProgressiveState {
        store: harness.store,
        demand,
        cache: harness.posts,
        network: harness.network,
        timing: Default::default(),
        capabilities: harness.capabilities,
        debug_feed: feed,
    });
    let router = configured_router_with_progressive_debug(
        GatewayRouterResources::new(HlsSessions::production(), gateway_fixture::media_client()),
        state,
        delivery,
        Arc::new(client_with_event_cache()),
    );
    let request = Request::put("/debug/api/focus")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"id":"b"}"#))
        .expect("request");

    let response = router.oneshot(request).await.expect("response");
    let DeliveryCommand::Focus(focus) = next_control(&mut commands).await else {
        panic!("expected focus");
    };

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(focus.current_index, 1);
}
