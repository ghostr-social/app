//! A blank search term still widens limits and skips the mp4 hunt, but
//! adds no NIP-50 term: widening uses the supplied value while the wire
//! filter uses its trimmed, non-blank form.

use serde_json::json;

use crate::query::video_filters::{discovery_filters, DiscoveryRequest};
use crate::tests::support::filter_json;

fn request() -> DiscoveryRequest {
    DiscoveryRequest {
        search_query: Some("   ".into()),
        ..DiscoveryRequest::default()
    }
}

#[test]
fn blank_term_still_skips_the_mp4_hunt() {
    assert_eq!(discovery_filters(&request()).len(), 3);
}

#[test]
fn blank_term_sets_no_search_field_anywhere() {
    for filter in discovery_filters(&request()) {
        assert!(filter_json(&filter).get("search").is_none());
    }
}

#[test]
fn blank_term_still_widens_the_video_limit() {
    let json = filter_json(&discovery_filters(&request())[0]);

    assert_eq!(json["limit"], json!(200));
}
