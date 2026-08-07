#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use nostr_sdk::{EventBuilder, Filter, Keys, Kind};
use rust_lib_ghostr::discovery::event_cache::client_with_event_cache;
use rust_lib_ghostr::video::debug_feed::{DebugFeed, DebugFeedStage};
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_progressive_debug;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::progressive_route::{ProgressiveState, ProgressiveTiming};
use std::sync::Arc;
use support::debug_clear::{hls_item, progressive_meta};
use support::delivery::start_harness;
use support::delivery_options::DeliveryOptions;
use tower::ServiceExt;

#[tokio::test]
async fn clear_api_removes_feed_download_hls_and_nostr_database_state() {
    let delivery = start_harness("ghostr-debug-clear", DeliveryOptions::default());
    let feed = DebugFeed::new(delivery.handle.clone(), Vec::new());
    feed.publish(1, DebugFeedStage::Settled, vec![hls_item()]);
    let hls = HlsSessions::production();
    let session = hls
        .acquire(vec!["https://media.example/live.m3u8".to_owned()])
        .await
        .expect("HLS session");
    delivery.cache.insert_video("stored", progressive_meta());
    delivery
        .store
        .write_range("stored", 0, &[7; 16])
        .await
        .expect("stored range");
    let client = Arc::new(client_with_event_cache());
    let event = EventBuilder::text_note("cached")
        .sign_with_keys(&Keys::generate())
        .expect("event");
    client
        .database()
        .save_event(&event)
        .await
        .expect("cache event");
    let state = Arc::new(ProgressiveState {
        store: delivery.store.clone(),
        demand: delivery.demand.clone(),
        cache: delivery.cache.clone(),
        network: delivery.network.clone(),
        timing: ProgressiveTiming::default(),
        debug_feed: feed.clone(),
    });
    let router = configured_router_with_progressive_debug(
        new_native_downloads(),
        hls.clone(),
        support::fixtures::trusted_media_client(),
        state,
        delivery.handle,
        client.clone(),
    );

    let response = router
        .oneshot(
            Request::delete("/debug/api/data")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("clear response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(delivery.store.used_bytes().await, 0);
    assert!(delivery.cache.videos().is_empty());
    assert_eq!(feed.snapshot().discovered_count, 0);
    assert!(hls.sources(&session).await.is_none());
    let events = client
        .database()
        .query(vec![Filter::new().kind(Kind::TextNote)])
        .await
        .expect("database query");
    assert!(events.is_empty());
}
