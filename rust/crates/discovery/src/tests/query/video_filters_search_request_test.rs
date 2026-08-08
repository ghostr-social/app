//! A viewer search term drops the mp4 note hunt, widens the video limit,
//! and carries the trimmed term on every query.

use serde_json::json;

use crate::query::video_filters::{discovery_filters, DiscoveryRequest};
use crate::tests::support::filter_json;

fn request() -> DiscoveryRequest {
    DiscoveryRequest {
        search_query: Some("  skate clips  ".into()),
        ..DiscoveryRequest::default()
    }
}

#[test]
fn search_request_builds_three_filters_without_the_hunt() {
    assert_eq!(discovery_filters(&request()).len(), 3);
}

#[test]
fn every_filter_carries_the_trimmed_viewer_term() {
    for filter in discovery_filters(&request()) {
        assert_eq!(filter_json(&filter)["search"], json!("skate clips"));
    }
}

#[test]
fn video_kind_filter_widens_to_the_discovery_limit() {
    let json = filter_json(&discovery_filters(&request())[0]);

    assert_eq!(json["kinds"], json!([21, 22, 34235, 34236]));
    assert_eq!(json["limit"], json!(200));
}

#[test]
fn file_filter_keeps_the_mime_scope_alongside_the_term() {
    let json = filter_json(&discovery_filters(&request())[2]);

    assert_eq!(json["kinds"], json!([1063]));
    assert_eq!(json["search"], json!("skate clips"));
    assert_eq!(json["#m"].as_array().map(Vec::len), Some(6));
}
