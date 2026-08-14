//! Tuple-safe deletion filters paired with their dependent repost wrappers.

use super::deletion_targets::{deletion_targets, DeletionTarget};
use crate::query::events::{plan_hinted_event_queries, HintedEventFilter};
use crate::query::search::QueryPlan;
use nostr_sdk::{Alphabet, Event, EventId, Filter, Kind, PublicKey, SingleLetterTag};
use std::collections::{BTreeMap, BTreeSet};

const TARGETS_PER_FILTER: usize = 100;
const DELETION_LIMIT: usize = 200;

pub(super) struct DependentDeletionPlan {
    pub(super) plan: QueryPlan,
    pub(super) dependencies: Vec<BTreeSet<EventId>>,
}

struct DeletionQuery {
    filter: Filter,
    hints: Vec<String>,
    dependents: BTreeSet<EventId>,
    rank: usize,
}

struct Evidence {
    dependents: BTreeSet<EventId>,
    rank: usize,
}

impl Default for Evidence {
    fn default() -> Self {
        Self {
            dependents: BTreeSet::new(),
            rank: usize::MAX,
        }
    }
}

#[cfg(test)]
pub(crate) fn deletion_plan(events: &[Event]) -> Option<QueryPlan> {
    dependent_deletion_plan(events).map(|dependent| dependent.plan)
}

pub(super) fn dependent_deletion_plan(events: &[Event]) -> Option<DependentDeletionPlan> {
    let targets = deletion_targets(events);
    let mut queries = tagged_queries(targets.events, Alphabet::E);
    queries.extend(tagged_queries(targets.addresses, Alphabet::A));
    queries.sort_by_key(|query| query.rank);
    (!queries.is_empty()).then(|| build_plan(queries))
}

fn build_plan(queries: Vec<DeletionQuery>) -> DependentDeletionPlan {
    let dependencies = queries
        .iter()
        .map(|query| query.dependents.clone())
        .collect();
    let filters = queries
        .into_iter()
        .map(|query| HintedEventFilter::new(query.filter, query.hints))
        .collect();
    DependentDeletionPlan {
        plan: plan_hinted_event_queries(filters),
        dependencies,
    }
}

fn tagged_queries(targets: Vec<DeletionTarget>, tag: Alphabet) -> Vec<DeletionQuery> {
    let mut grouped = BTreeMap::<(PublicKey, Vec<String>), BTreeMap<String, Evidence>>::new();
    for target in targets {
        let evidence = grouped
            .entry((target.author, target.hints))
            .or_default()
            .entry(target.value)
            .or_default();
        evidence.dependents.extend(target.dependents);
        evidence.rank = evidence.rank.min(target.rank);
    }
    grouped
        .into_iter()
        .flat_map(|((author, hints), values)| grouped_queries(author, hints, values, tag))
        .collect()
}

fn grouped_queries(
    author: PublicKey,
    hints: Vec<String>,
    values: BTreeMap<String, Evidence>,
    tag: Alphabet,
) -> Vec<DeletionQuery> {
    let values: Vec<_> = values.into_iter().collect();
    values
        .chunks(TARGETS_PER_FILTER)
        .map(|chunk| query(author, hints.clone(), chunk, tag))
        .collect()
}

fn query(
    author: PublicKey,
    hints: Vec<String>,
    values: &[(String, Evidence)],
    tag: Alphabet,
) -> DeletionQuery {
    let dependents = values
        .iter()
        .flat_map(|(_, evidence)| evidence.dependents.iter().copied())
        .collect();
    let rank = values
        .iter()
        .map(|(_, evidence)| evidence.rank)
        .min()
        .unwrap_or(usize::MAX);
    let targets = values.iter().map(|(value, _)| value.clone());
    let filter = deletion_filter(author).custom_tag(SingleLetterTag::lowercase(tag), targets);
    DeletionQuery {
        filter,
        hints,
        dependents,
        rank,
    }
}

fn deletion_filter(author: PublicKey) -> Filter {
    Filter::new()
        .kind(Kind::EventDeletion)
        .author(author)
        .limit(DELETION_LIMIT)
}
