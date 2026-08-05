//! Generic Nostr read plans: every filter is required, so one failure
//! fails the batch instead of narrowing it like additive feed queries.

use crate::discovery::event_cache::ViewerScope;
use crate::discovery::search_queries::{
    is_discovery, target_for, OutboxLookup, OutboxRoute, PlannedQuery, QueryPlan, QueryRole,
    DISCOVERY_QUERY_TIMEOUT, FEED_QUERY_TIMEOUT,
};
use nostr_sdk::{Filter, PublicKey};
use std::collections::BTreeSet;

pub fn plan_event_queries(filters: Vec<Filter>) -> QueryPlan {
    QueryPlan {
        outbox: OutboxLookup::DiscoveryRelays,
        queries: filters.into_iter().map(planned_query).collect(),
        viewer: ViewerScope::Unknown,
    }
}

fn planned_query(filter: Filter) -> PlannedQuery {
    let outbox = OutboxRoute::Filter(event_outbox(&filter));
    let timeout = if is_discovery(&filter) {
        DISCOVERY_QUERY_TIMEOUT
    } else {
        FEED_QUERY_TIMEOUT
    };
    PlannedQuery {
        target: target_for(&filter),
        role: QueryRole::Primary,
        timeout,
        filter,
        outbox,
    }
}

fn event_outbox(filter: &Filter) -> OutboxLookup {
    let authors = query_authors(filter);
    if authors.is_empty() {
        OutboxLookup::DiscoveryRelays
    } else {
        OutboxLookup::AuthorWriteRelays(authors)
    }
}

fn query_authors(filter: &Filter) -> Vec<PublicKey> {
    filter
        .authors
        .iter()
        .flatten()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
