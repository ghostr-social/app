//! Bounded, target-fair filters for NIP-18 target enrichment.

use super::target_hints::{build_plan, normalized_hints, TargetFilter, MAX_HINTS_PER_TARGET};
use crate::content::repost_reference::{lookup_for_enrichment, RepostLookup, RepostLookupTarget};
use crate::feed::store::FEED_POST_RETENTION;
use crate::query::search::QueryPlan;
use nostr_sdk::{Event, EventId, Filter, Kind, PublicKey, Timestamp};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

const TARGETS_PER_FILTER: usize = 8;
pub(crate) const MAX_TARGET_LOOKUPS: usize = FEED_POST_RETENTION;

type EventGroup = (Option<PublicKey>, Option<u16>);
type CoordinateGroup = (PublicKey, u16);
type TargetValues<T> = BTreeMap<T, BTreeSet<String>>;

struct RankedLookup {
    lookup: RepostLookup,
    created_at: Timestamp,
    wrapper_id: EventId,
}

pub(crate) fn target_plan(events: &[Event]) -> Option<QueryPlan> {
    let lookups = unique_lookups(events);
    let (event_groups, coordinate_groups) = grouped_targets(lookups);
    let mut targets = event_filters(event_groups);
    targets.extend(coordinate_filters(coordinate_groups));
    build_plan(targets)
}

fn unique_lookups(events: &[Event]) -> Vec<RepostLookup> {
    let mut unique = BTreeMap::<RepostLookupTarget, RankedLookup>::new();
    for event in events {
        if let Some(lookup) = ranked_lookup(event) {
            insert_lookup(&mut unique, lookup);
        }
    }
    let mut ranked: Vec<_> = unique.into_values().collect();
    ranked.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| left.wrapper_id.cmp(&right.wrapper_id))
    });
    ranked.truncate(MAX_TARGET_LOOKUPS);
    ranked.into_iter().map(|ranked| ranked.lookup).collect()
}

fn ranked_lookup(event: &Event) -> Option<RankedLookup> {
    let mut lookup = lookup_for_enrichment(event)?;
    lookup.relay_hints = normalized_hints(lookup.relay_hints);
    Some(RankedLookup {
        lookup,
        created_at: event.created_at,
        wrapper_id: event.id,
    })
}

fn insert_lookup(unique: &mut BTreeMap<RepostLookupTarget, RankedLookup>, incoming: RankedLookup) {
    match unique.entry(incoming.lookup.target.clone()) {
        Entry::Vacant(entry) => {
            entry.insert(incoming);
        }
        Entry::Occupied(mut entry) => entry.get_mut().merge(incoming),
    }
}

impl RankedLookup {
    fn merge(&mut self, incoming: Self) {
        let is_newer = incoming.is_newer_than(self);
        merge_hints(&mut self.lookup.relay_hints, incoming.lookup.relay_hints);
        if is_newer {
            self.created_at = incoming.created_at;
            self.wrapper_id = incoming.wrapper_id;
        }
    }

    fn is_newer_than(&self, other: &Self) -> bool {
        self.created_at > other.created_at
            || (self.created_at == other.created_at && self.wrapper_id < other.wrapper_id)
    }
}

fn merge_hints(current: &mut Vec<String>, incoming: Vec<String>) {
    for hint in incoming {
        if current.len() >= MAX_HINTS_PER_TARGET {
            return;
        }
        if !current.contains(&hint) {
            current.push(hint);
        }
    }
}

fn grouped_targets(
    lookups: Vec<RepostLookup>,
) -> (
    BTreeMap<EventGroup, TargetValues<EventId>>,
    BTreeMap<CoordinateGroup, TargetValues<String>>,
) {
    let mut events = BTreeMap::new();
    let mut coordinates = BTreeMap::new();
    for lookup in lookups {
        match lookup.target {
            RepostLookupTarget::Event { id, author, kind } => {
                insert_target(&mut events, (author, kind), id, lookup.relay_hints);
            }
            RepostLookupTarget::Coordinate {
                author,
                kind,
                identifier,
            } => {
                insert_target(
                    &mut coordinates,
                    (author, kind),
                    identifier,
                    lookup.relay_hints,
                );
            }
        }
    }
    (events, coordinates)
}

fn insert_target<K: Ord, V: Ord>(
    groups: &mut BTreeMap<K, TargetValues<V>>,
    group: K,
    value: V,
    hints: Vec<String>,
) {
    groups
        .entry(group)
        .or_default()
        .entry(value)
        .or_default()
        .extend(hints);
}

fn event_filters(groups: BTreeMap<EventGroup, TargetValues<EventId>>) -> Vec<TargetFilter> {
    groups
        .into_iter()
        .flat_map(|((author, kind), values)| event_group(author, kind, values))
        .collect()
}

fn event_group(
    author: Option<PublicKey>,
    kind: Option<u16>,
    values: TargetValues<EventId>,
) -> Vec<TargetFilter> {
    chunks(values)
        .into_iter()
        .map(|chunk| {
            let mut filter = Filter::new().ids(chunk.iter().map(|(id, _)| *id));
            if let Some(author) = author {
                filter = filter.author(author);
            }
            if let Some(kind) = kind {
                filter = filter.kind(Kind::from(kind));
            }
            TargetFilter::new(filter, &chunk)
        })
        .collect()
}

fn coordinate_filters(
    groups: BTreeMap<CoordinateGroup, TargetValues<String>>,
) -> Vec<TargetFilter> {
    groups
        .into_iter()
        .flat_map(|((author, kind), values)| coordinate_group(author, kind, values))
        .collect()
}

fn coordinate_group(
    author: PublicKey,
    kind: u16,
    values: TargetValues<String>,
) -> Vec<TargetFilter> {
    chunks(values)
        .into_iter()
        .map(|chunk| {
            let filter = Filter::new()
                .author(author)
                .kind(Kind::from(kind))
                .identifiers(chunk.iter().map(|(identifier, _)| identifier.clone()));
            TargetFilter::new(filter, &chunk)
        })
        .collect()
}

fn chunks<T: Clone + Ord>(values: TargetValues<T>) -> Vec<Vec<(T, BTreeSet<String>)>> {
    let values: Vec<_> = values.into_iter().collect();
    values
        .chunks(TARGETS_PER_FILTER)
        .map(<[_]>::to_vec)
        .collect()
}
