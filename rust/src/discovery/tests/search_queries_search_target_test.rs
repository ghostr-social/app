//! NIP-50 terms only work on relays that index for search: a viewer term
//! routes every query to the search relay set and skips the outbox
//! entirely (lib/platform/nostr/ndk_nostr_video_event_query.dart
//! `_targets` + `_outboxRelays`).

use crate::discovery::search_queries::{plan_discovery, OutboxLookup, RelayTarget};
use crate::discovery::video_filters::DiscoveryRequest;

fn plan() -> crate::discovery::search_queries::QueryPlan {
    plan_discovery(&DiscoveryRequest {
        search_query: Some("skate clips".into()),
        ..DiscoveryRequest::default()
    })
}

#[test]
fn every_search_query_targets_the_search_relays() {
    let targets: Vec<RelayTarget> =
        plan().queries.iter().map(|query| query.target.clone()).collect();

    assert_eq!(targets, vec![RelayTarget::SearchRelays; 3]);
}

#[test]
fn a_viewer_term_never_needs_the_outbox() {
    assert_eq!(plan().outbox, OutboxLookup::Skip);
}
