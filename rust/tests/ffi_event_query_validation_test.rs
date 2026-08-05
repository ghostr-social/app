//! Invalid and empty query requests resolve at the FFI boundary.

use rust_lib_ghostr::api::event_control::{ffi_query_events, ffi_query_events_batch};
use rust_lib_ghostr::api::event_types::{FfiNostrEventFilter, FfiNostrTagFilter};

fn filter() -> FfiNostrEventFilter {
    FfiNostrEventFilter {
        kinds: vec![7],
        authors: Vec::new(),
        event_tags: Vec::new(),
        tag_filters: Vec::new(),
        limit: 25,
        until: None,
        search: None,
    }
}

#[tokio::test]
async fn zero_limit_is_rejected_before_engine_lookup() {
    let error = ffi_query_events(FfiNostrEventFilter {
        limit: 0,
        ..filter()
    })
    .await
    .expect_err("zero limit");

    assert!(error.to_string().contains("limit must be positive"));
}

#[tokio::test]
async fn non_letter_tag_is_rejected_before_engine_lookup() {
    let error = ffi_query_events(FfiNostrEventFilter {
        tag_filters: vec![FfiNostrTagFilter {
            name: "topic".to_owned(),
            values: vec!["rust".to_owned()],
        }],
        ..filter()
    })
    .await
    .expect_err("invalid tag");

    assert!(error.to_string().contains("one letter"));
}

#[tokio::test]
async fn empty_batch_returns_empty_before_engine_lookup() {
    assert!(ffi_query_events_batch(Vec::new())
        .await
        .expect("empty batch")
        .is_empty());
}

#[tokio::test]
async fn batch_over_twenty_filters_is_rejected() {
    let error = ffi_query_events_batch(vec![filter(); 21])
        .await
        .expect_err("oversized batch");

    assert!(error.to_string().contains("exceeds 20"));
}
