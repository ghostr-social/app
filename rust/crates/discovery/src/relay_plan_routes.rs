//! Resolves each planned query's outbox route under its own relay cap.

use crate::outbox_directory::OutboxDirectory;
use crate::search_queries::{OutboxLookup, OutboxRoute, QueryPlan};

pub(crate) fn plan_outbox_relays(
    directory: &OutboxDirectory,
    plan: &QueryPlan,
    cap: usize,
) -> Vec<Option<Vec<String>>> {
    let shared = outbox_relays(directory, &plan.outbox, cap);
    plan.queries
        .iter()
        .map(|query| match &query.outbox {
            OutboxRoute::Shared => shared.clone(),
            OutboxRoute::Filter(lookup) => outbox_relays(directory, lookup, cap),
        })
        .collect()
}

pub(crate) fn outbox_relays(
    directory: &OutboxDirectory,
    lookup: &OutboxLookup,
    cap: usize,
) -> Option<Vec<String>> {
    let relays = match lookup {
        OutboxLookup::Skip => return None,
        OutboxLookup::DiscoveryRelays => directory.discovery_relays(cap),
        OutboxLookup::AuthorWriteRelays(authors) => directory.relays_for_authors(authors, cap),
    };
    (!relays.is_empty()).then_some(relays)
}
