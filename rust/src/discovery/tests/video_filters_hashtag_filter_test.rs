//! Hashtag requests put the expanded `#t` values on every query, widen the
//! video limit, and keep the mp4 hunt because no viewer term is present.

use serde_json::json;

use crate::discovery::tests::support::filter_json;
use crate::discovery::video_filters::{discovery_filters, DiscoveryRequest};

fn request() -> DiscoveryRequest {
    DiscoveryRequest {
        hashtags: vec!["surf".into()],
        ..DiscoveryRequest::default()
    }
}

#[test]
fn hashtags_keep_all_four_queries() {
    assert_eq!(discovery_filters(&request()).len(), 4);
}

#[test]
fn every_filter_carries_the_case_variant_tag_values() {
    for filter in discovery_filters(&request()) {
        // nostr_sdk stores tag values as an ordered set.
        assert_eq!(filter_json(&filter)["#t"], json!(["SURF", "Surf", "surf"]));
    }
}

#[test]
fn hashtags_widen_the_video_limit() {
    let json = filter_json(&discovery_filters(&request())[0]);

    assert_eq!(json["limit"], json!(200));
}

#[test]
fn file_filter_keeps_mimes_alongside_hashtags() {
    let json = filter_json(&discovery_filters(&request())[3]);

    assert_eq!(json["#m"].as_array().map(Vec::len), Some(6));
    assert_eq!(json["#t"].as_array().map(Vec::len), Some(3));
}
