//! The viewer scope rides the plan the executor runs, never the wire
//! filter — like `routing_authors`, it is routing and bookkeeping, not
//! something a relay is asked about. Bootstrap plans name nobody, so a
//! relay-list chase never rescopes the pool.

use crate::cache::ViewerScope;
use crate::outbox::plans::viewer_lists_plan;
use crate::query::search::plan_discovery;
use crate::tests::support::{author, filter_json, AUTHOR_A};
use crate::query::video_filters::DiscoveryRequest;

fn request() -> DiscoveryRequest {
    DiscoveryRequest {
        viewer: ViewerScope::SignedIn(author(AUTHOR_A)),
        ..DiscoveryRequest::default()
    }
}

#[test]
fn the_plan_carries_the_requests_viewer_scope() {
    assert_eq!(
        plan_discovery(&request()).viewer,
        ViewerScope::SignedIn(author(AUTHOR_A))
    );
}

#[test]
fn the_viewer_scope_never_reaches_a_wire_filter() {
    for query in plan_discovery(&request()).queries {
        let json = filter_json(&query.filter);
        assert!(
            json.get("authors").is_none(),
            "the scoped viewer must not filter the query: {json}"
        );
    }
}

#[test]
fn a_relay_list_chase_leaves_the_scope_alone() {
    assert_eq!(
        viewer_lists_plan(author(AUTHOR_A)).viewer,
        ViewerScope::Unknown
    );
}
