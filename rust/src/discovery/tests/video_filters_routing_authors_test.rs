//! Routing-only authors choose relays without narrowing the wire filter,
//! so those relays' whole catalogue remains eligible.

use crate::discovery::tests::support::{author, filter_json, AUTHOR_A, AUTHOR_B};
use crate::discovery::video_filters::{discovery_filters, DiscoveryRequest};

#[test]
fn routing_authors_never_reach_the_wire_filter() {
    let request = DiscoveryRequest {
        routing_authors: vec![author(AUTHOR_A), author(AUTHOR_B)],
        ..DiscoveryRequest::default()
    };

    for filter in discovery_filters(&request) {
        let json = filter_json(&filter);
        assert!(
            json.get("authors").is_none(),
            "a routing-only author must not scope the query: {json}"
        );
    }
}

/// A scoped request keeps filtering by its own authors; the routing set
/// only ever decides where the query is sent.
#[test]
fn filtering_authors_still_scope_the_wire_filter() {
    let request = DiscoveryRequest {
        authors: vec![author(AUTHOR_A)],
        routing_authors: vec![author(AUTHOR_B)],
        ..DiscoveryRequest::default()
    };

    let json = filter_json(&discovery_filters(&request)[0]);

    assert_eq!(json["authors"], serde_json::json!([AUTHOR_A]));
}
