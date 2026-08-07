//! Every generic batch filter is required and keeps the routing it would
//! receive as a standalone query.

use crate::event_queries::plan_event_queries;
use crate::search_queries::{
    OutboxLookup, OutboxRoute, QueryRole, DISCOVERY_QUERY_TIMEOUT,
};
use nostr_sdk::{Filter, Keys, Kind};

#[test]
fn mixed_batch_keeps_each_filters_standalone_outbox_route() {
    let first = Keys::generate().public_key();
    let second = Keys::generate().public_key();
    let third = Keys::generate().public_key();
    let mut pair = vec![first, second];
    pair.sort();
    let plan = plan_event_queries(vec![
        Filter::new().kind(Kind::Reaction).authors(pair.clone()),
        Filter::new().kind(Kind::Comment).author(third),
        Filter::new().kind(Kind::TextNote),
    ]);

    assert_eq!(
        plan.queries[0].outbox,
        OutboxRoute::Filter(OutboxLookup::AuthorWriteRelays(pair))
    );
    assert_eq!(
        plan.queries[1].outbox,
        OutboxRoute::Filter(OutboxLookup::AuthorWriteRelays(vec![third]))
    );
    assert_eq!(
        plan.queries[2].outbox,
        OutboxRoute::Filter(OutboxLookup::DiscoveryRelays)
    );
    assert!(plan
        .queries
        .iter()
        .all(|query| query.role == QueryRole::Primary),);
}

#[test]
fn generic_tag_filter_keeps_the_discovery_timeout() {
    let plan = plan_event_queries(vec![Filter::new().hashtag("surf")]);

    assert_eq!(plan.queries[0].timeout, DISCOVERY_QUERY_TIMEOUT);
}
