#![cfg(feature = "video-debug-web")]

mod support;

use ghostr_delivery::debug::feed::DebugFeedStage;
use ghostr_delivery::delivery_events::command_channel;
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_engine::adaptive::DiscoveryDemand;
use nostr_sdk::{EventBuilder, Keys, Kind, Timestamp};
use rust_lib_ghostr::api::debug::nostr::{DebugNostrConfiguration, DebugNostrRuntime};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

#[tokio::test]
async fn nostr_discovery_failure_reaches_the_debug_feed() {
    let client = Arc::new(client_with_event_cache());
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Expand);
    let (delivery, _commands) = command_channel();
    let feed = ghostr_delivery::debug::feed::DebugFeed::new(delivery, Vec::new());
    let _nostr = DebugNostrRuntime::start(
        client,
        demand,
        DebugNostrConfiguration {
            read_relays: Vec::new(),
            search_relays: Vec::new(),
        },
        feed.clone(),
    )
    .await
    .expect("Nostr runtime");

    tokio::time::timeout(Duration::from_secs(2), async {
        while feed.snapshot().stage != DebugFeedStage::Failed {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("failed feed stage");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consecutive_relay_events_keep_advancing_the_debug_feed() {
    let keys = Keys::generate();
    let events = (1..=100).map(|sequence| {
        EventBuilder::new(
            Kind::TextNote,
            format!("https://cdn.example/{sequence}.mp4"),
        )
        .custom_created_at(Timestamp::from(sequence))
        .sign_with_keys(&keys)
        .expect("video event")
    });
    let relay = support::nostr_relay::relay_serving(events.collect()).await;
    let client = Arc::new(client_with_event_cache());
    let (_demand_sender, demand) = watch::channel(DiscoveryDemand::Expand);
    let (delivery, _commands) = command_channel();
    let feed = ghostr_delivery::debug::feed::DebugFeed::new(delivery, vec![relay.clone()]);

    let _runtime = DebugNostrRuntime::start(
        client,
        demand,
        DebugNostrConfiguration {
            read_relays: vec![relay.clone()],
            search_relays: vec![relay],
        },
        feed.clone(),
    )
    .await
    .expect("Nostr runtime");

    tokio::time::timeout(Duration::from_secs(3), async {
        while feed.snapshot().revision < 100 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("every relay event reaches the projection");
    assert_eq!(feed.snapshot().discovered_count, 100);
}
