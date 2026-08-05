//! Generic reads travel through the running Rust discovery scheduler.

mod support;

use rust_lib_ghostr::api::event_control::ffi_query_events;
use rust_lib_ghostr::api::event_types::FfiNostrEventFilter;
use support::fixtures::temp_directory;

#[tokio::test]
async fn a_query_without_configured_relays_reports_the_runtime_failure() {
    let directory = temp_directory("ghostr-event-query-no-relays");
    support::engine::start(&directory, 1024)
        .await
        .expect("engine start");

    let error = ffi_query_events(FfiNostrEventFilter {
        kinds: vec![7],
        authors: Vec::new(),
        event_tags: Vec::new(),
        tag_filters: Vec::new(),
        limit: 1,
        until: None,
        search: None,
    })
    .await
    .expect_err("query should require a relay");

    assert!(error.to_string().contains("no Nostr relays are configured"));
    std::fs::remove_dir_all(directory).expect("remove cache");
}
