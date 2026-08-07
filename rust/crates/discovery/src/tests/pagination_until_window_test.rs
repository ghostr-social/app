//! Older pages set the inclusive `until` cutoff on every query; NIP-01
//! matches events whose `created_at` is at or below that boundary.

use nostr_sdk::Timestamp;
use serde_json::json;

use crate::tests::support::filter_json;
use crate::video_filters::{discovery_filters, DiscoveryRequest};

const CURSOR_SECS: u64 = 1_722_000_000;

fn request() -> DiscoveryRequest {
    DiscoveryRequest {
        older_than: Some(Timestamp::from(CURSOR_SECS)),
        ..DiscoveryRequest::default()
    }
}

#[test]
fn every_filter_carries_the_inclusive_until_cutoff() {
    let filters = discovery_filters(&request());

    assert_eq!(filters.len(), 4);
    for filter in filters {
        assert_eq!(filter_json(&filter)["until"], json!(CURSOR_SECS));
    }
}

#[test]
fn first_pages_carry_no_until_cutoff() {
    for filter in discovery_filters(&DiscoveryRequest::default()) {
        assert!(filter_json(&filter).get("until").is_none());
    }
}

#[test]
fn pagination_keeps_the_same_limits_as_the_first_page() {
    let filters = discovery_filters(&request());

    assert_eq!(filter_json(&filters[0])["limit"], json!(80));
    assert_eq!(filter_json(&filters[1])["limit"], json!(200));
}
