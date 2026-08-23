//! Generic FFI reads never present cache-only rows as an authoritative result.

use crate::api::engine_control::{ffi_start_engine, FfiDataUsageLevel, FfiEngineConfiguration};
use crate::api::event_control::ffi_query_events;
use crate::api::event_types::FfiNostrEventFilter;
use crate::api::network_control::FfiDeliveryNetworkStatus;
use crate::api::runtime::registry;
use nostr_sdk::{EventBuilder, Keys, Kind};

#[tokio::test]
async fn generic_query_rejects_a_cache_only_partial_result() {
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

    let error = ffi_query_events(FfiNostrEventFilter {
        kinds: vec![Kind::Reaction.as_u16()],
        authors: Vec::new(),
        event_tags: Vec::new(),
        tag_filters: Vec::new(),
        limit: 1,
        until: None,
        search: None,
    })
    .await
    .expect_err("cache fallback is not an authoritative relay answer");

    assert!(error
        .to_string()
        .contains("did not complete authoritatively"));
    std::fs::remove_dir_all(directory).expect("remove cache");
}
