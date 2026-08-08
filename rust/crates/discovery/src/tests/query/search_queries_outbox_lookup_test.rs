//! Outbox relays serve requests that carry no viewer term: the directory's
//! discovery relays when unscoped, and the authors' write relays when scoped.

use crate::query::search::{plan_discovery, OutboxLookup};
use crate::query::video_filters::DiscoveryRequest;
use crate::tests::support::{author, AUTHOR_A};

#[test]
fn unscoped_requests_look_up_the_discovery_relays() {
    let plan = plan_discovery(&DiscoveryRequest::default());

    assert_eq!(plan.outbox, OutboxLookup::DiscoveryRelays);
}

#[test]
fn author_requests_look_up_their_write_relays() {
    let plan = plan_discovery(&DiscoveryRequest {
        authors: vec![author(AUTHOR_A)],
        ..DiscoveryRequest::default()
    });

    assert_eq!(
        plan.outbox,
        OutboxLookup::AuthorWriteRelays(vec![author(AUTHOR_A)])
    );
}

#[test]
fn a_blank_term_still_uses_the_outbox() {
    // Dart's `_outboxRelays` checks the normalized primary search, which
    // turns a blank term into null — the lookup still happens.
    let plan = plan_discovery(&DiscoveryRequest {
        search_query: Some("   ".into()),
        ..DiscoveryRequest::default()
    });

    assert_eq!(plan.outbox, OutboxLookup::DiscoveryRelays);
}
