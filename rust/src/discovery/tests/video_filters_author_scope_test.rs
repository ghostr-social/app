//! Author-scoped requests carry the wanted authors on every query
//! (lib/platform/nostr/video_discovery_queries.dart: `scope.authors` from
//! `authorPublicKeys` lands on each built query).

use serde_json::json;

use crate::discovery::tests::support::{author, filter_json, AUTHOR_A, AUTHOR_B};
use crate::discovery::video_filters::{discovery_filters, DiscoveryRequest};

fn request() -> DiscoveryRequest {
    DiscoveryRequest {
        authors: vec![author(AUTHOR_A), author(AUTHOR_B)],
        ..DiscoveryRequest::default()
    }
}

#[test]
fn every_filter_carries_the_author_scope() {
    let filters = discovery_filters(&request());

    assert_eq!(filters.len(), 4);
    for filter in filters {
        assert_eq!(filter_json(&filter)["authors"], json!([AUTHOR_A, AUTHOR_B]));
    }
}

#[test]
fn author_scope_keeps_the_narrow_video_limit() {
    // Authors alone do not widen the request (wide = term or hashtags).
    let json = filter_json(&discovery_filters(&request())[0]);

    assert_eq!(json["limit"], json!(80));
}
