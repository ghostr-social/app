//! Optional generic-query fields stay absent when Dart does not supply them.

use nostr_sdk::Filter;
use rust_lib_ghostr::api::event_types::FfiNostrEventFilter;

#[test]
fn ffi_filter_omits_absent_until_and_search_fields() {
    let input = FfiNostrEventFilter {
        kinds: vec![7],
        authors: Vec::new(),
        event_tags: Vec::new(),
        tag_filters: Vec::new(),
        limit: 25,
        until: None,
        search: None,
    };

    let wire = serde_json::to_value(Filter::try_from(input).expect("valid filter"))
        .expect("filter serializes");

    assert!(wire.get("until").is_none());
    assert!(wire.get("search").is_none());
}
