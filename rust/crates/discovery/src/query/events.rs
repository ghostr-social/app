//! Generic Nostr read plans: every filter is required, so one failure
//! fails the batch instead of narrowing it like additive feed queries.

use crate::cache::ViewerScope;
use crate::query::search::{
    is_discovery, target_for, OutboxLookup, OutboxRoute, PlannedQuery, QueryPlan, QueryRole,
    RelayTarget, DISCOVERY_QUERY_TIMEOUT, FEED_QUERY_TIMEOUT,
};
use nostr_sdk::{Filter, PublicKey};
use std::collections::BTreeSet;

pub fn plan_event_queries(filters: Vec<Filter>) -> QueryPlan {
    let filters = filters
        .into_iter()
        .map(|filter| HintedEventFilter::new(filter, Vec::new()))
        .collect();
    plan_hinted_event_queries(filters)
}

pub(crate) struct HintedEventFilter {
    filter: Filter,
    hints: Vec<String>,
}

impl HintedEventFilter {
    pub(crate) fn new(filter: Filter, hints: Vec<String>) -> Self {
        Self { filter, hints }
    }
}

pub(crate) fn plan_hinted_event_queries(filters: Vec<HintedEventFilter>) -> QueryPlan {
    QueryPlan {
        outbox: OutboxLookup::DiscoveryRelays,
        queries: filters.into_iter().map(hinted_query).collect(),
        viewer: ViewerScope::Unknown,
    }
}

fn hinted_query(input: HintedEventFilter) -> PlannedQuery {
    let mut query = planned_query(input.filter);
    if !input.hints.is_empty() {
        query.target = RelayTarget::HintedRelays(input.hints);
    }
    query
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
