//! Generic FFI reads return locally admitted rows while relays are unavailable.

use crate::api::engine_control::{ffi_start_engine, FfiDataUsageLevel, FfiEngineConfiguration};
use crate::api::event_control::ffi_query_events;
use crate::api::event_types::FfiNostrEventFilter;
use crate::api::network_control::FfiDeliveryNetworkStatus;
use crate::api::runtime::registry;
use nostr_sdk::{EventBuilder, Keys, Kind};

#[tokio::test]
async fn generic_query_returns_an_accepted_event_from_the_session_pool() {
    let event = EventBuilder::new(Kind::Reaction, "+")
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let directory = std::env::temp_dir().join(format!("ghostr-event-query-{}", event.id));
    ffi_start_engine(
        directory.to_string_lossy().to_string(),
        FfiEngineConfiguration {
            read_relay_urls: Vec::new(),
            search_relay_urls: Vec::new(),
            data_usage: FfiDataUsageLevel::Balanced,
            max_storage_bytes: 1024,
        },
        None,
        FfiDeliveryNetworkStatus::unavailable(),
    )
    .await
    .expect("engine start");
    let engine = registry::engine().expect("running engine");
    let session = engine.discovery.session_generation();
    engine.discovery.remember_accepted(session, &event).await;

    let found = ffi_query_events(FfiNostrEventFilter {
        kinds: vec![Kind::Reaction.as_u16()],
        authors: Vec::new(),
        event_tags: Vec::new(),
        tag_filters: Vec::new(),
        limit: 1,
        until: None,
        search: None,
    })
    .await
    .expect("warm query");

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, event.id.to_hex());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
