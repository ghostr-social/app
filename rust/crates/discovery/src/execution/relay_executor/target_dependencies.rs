//! Wrapper identities carried alongside each target-enrichment query.

use super::target_planning::target_plan;
use crate::content::repost_reference::{lookup_for_enrichment, RepostLookupTarget};
use crate::query::search::{PlannedQuery, QueryPlan};
use nostr_sdk::{Alphabet, Event, EventId, Kind, SingleLetterTag};
use std::collections::BTreeSet;

pub(super) struct DependentTargetPlan {
    pub(super) plan: QueryPlan,
    pub(super) dependencies: Vec<BTreeSet<EventId>>,
    pub(super) unplanned: BTreeSet<EventId>,
}

pub(super) fn dependent_target_plan(events: &[Event]) -> Option<DependentTargetPlan> {
    let plan = target_plan(events)?;
    let mut dependencies = vec![BTreeSet::new(); plan.queries.len()];
    let mut unplanned = BTreeSet::new();
    for event in events {
        let Some(lookup) = lookup_for_enrichment(event) else {
            continue;
        };
        match matching_query(&plan.queries, &lookup.target) {
            Some(index) => {
                dependencies[index].insert(event.id);
            }
            None => {
                unplanned.insert(event.id);
            }
        }
    }
    Some(DependentTargetPlan {
        plan,
        dependencies,
        unplanned,
    })
}

fn matching_query(queries: &[PlannedQuery], target: &RepostLookupTarget) -> Option<usize> {
    queries.iter().position(|query| match target {
        RepostLookupTarget::Event { id, author, kind } => {
            event_matches(query, id, author.as_ref(), *kind)
        }
        RepostLookupTarget::Coordinate {
            author,
            kind,
            identifier,
        } => coordinate_matches(query, author, *kind, identifier),
    })
}

fn event_matches(
    query: &PlannedQuery,
    id: &EventId,
    author: Option<&nostr_sdk::PublicKey>,
    kind: Option<u16>,
) -> bool {
    id_matches(query, id)
        && author.is_none_or(|author| author_matches(query, author))
        && kind.is_none_or(|kind| kind_matches(query, kind))
}

fn id_matches(query: &PlannedQuery, id: &EventId) -> bool {
    query
        .filter
        .ids
        .as_ref()
        .is_some_and(|ids| ids.contains(id))
}

fn coordinate_matches(
    query: &PlannedQuery,
    author: &nostr_sdk::PublicKey,
    kind: u16,
    identifier: &str,
) -> bool {
    author_matches(query, author)
        && kind_matches(query, kind)
        && identifier_matches(query, identifier)
}

fn author_matches(query: &PlannedQuery, author: &nostr_sdk::PublicKey) -> bool {
    query
        .filter
        .authors
        .as_ref()
        .is_some_and(|authors| authors.contains(author))
}

fn kind_matches(query: &PlannedQuery, kind: u16) -> bool {
    query
        .filter
        .kinds
        .as_ref()
        .is_some_and(|kinds| kinds.contains(&Kind::from(kind)))
}

fn identifier_matches(query: &PlannedQuery, identifier: &str) -> bool {
    let tag = SingleLetterTag::lowercase(Alphabet::D);
    query
        .filter
        .generic_tags
        .get(&tag)
        .is_some_and(|values| values.contains(identifier))
}

#[cfg(test)]
#[path = "target_dependencies_axiom_test.rs"]
pub(crate) mod axiom_test_support;
