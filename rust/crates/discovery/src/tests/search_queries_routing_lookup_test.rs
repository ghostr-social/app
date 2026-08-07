//! A request's routing set decides the outbox lookup: routing-only
//! authors send the plan to their write relays even though no query
//! filters by them.

use crate::search_queries::{plan_discovery, OutboxLookup};
use crate::tests::support::{author, AUTHOR_A, AUTHOR_B};
use crate::video_filters::DiscoveryRequest;

#[test]
fn routing_authors_pick_their_write_relays() {
    let plan = plan_discovery(&DiscoveryRequest {
        routing_authors: vec![author(AUTHOR_A)],
        ..DiscoveryRequest::default()
    });

    assert_eq!(
        plan.outbox,
        OutboxLookup::AuthorWriteRelays(vec![author(AUTHOR_A)])
    );
}

/// Scoped requests (profile grids) keep routing by the authors they
/// filter by; the routing set never overrides them.
#[test]
fn a_scoped_request_routes_by_its_own_authors() {
    let plan = plan_discovery(&DiscoveryRequest {
        authors: vec![author(AUTHOR_A)],
        routing_authors: vec![author(AUTHOR_B)],
        ..DiscoveryRequest::default()
    });

    assert_eq!(
        plan.outbox,
        OutboxLookup::AuthorWriteRelays(vec![author(AUTHOR_A)])
    );
}

/// Nothing known to route by falls back to the directory's own
/// discovery relays (bootstrap until the viewer's follows land).
#[test]
fn an_unrouted_request_falls_back_to_the_discovery_relays() {
    let plan = plan_discovery(&DiscoveryRequest::default());

    assert_eq!(plan.outbox, OutboxLookup::DiscoveryRelays);
}
