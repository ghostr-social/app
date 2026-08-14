//! Bounded repost settlement before raw wrappers reach feed projection.

use super::repost_support::{exact_target_id, verified_event_ids, ResolvedRepostSupport};
use crate::content::repost_reference::{lookup_for_enrichment, reference_for_repost};
use crate::content::reposts::feed_post_from_event;
use crate::plan_executor::RepostRetryDelta;
use crate::scheduler::deferred_reposts::MAX_REPOSTS_PER_ATTEMPT;
use nostr_sdk::{Event, EventId, Kind};
use std::collections::{BTreeSet, HashSet};

#[derive(Default)]
pub(super) struct RepostSettlement {
    enabled: bool,
    base_ids: HashSet<EventId>,
    considered: Vec<EventId>,
    selected: Vec<Event>,
    deferred: Vec<Event>,
    target_settled: BTreeSet<EventId>,
    retry: BTreeSet<EventId>,
    support: ResolvedRepostSupport,
}

struct TargetStatus {
    id: EventId,
    materialized: bool,
    exact: bool,
    exact_present: bool,
    needs_lookup: bool,
}

impl RepostSettlement {
    pub(super) fn prepare(
        base: Vec<Event>,
        pending: Vec<Event>,
        enabled: bool,
    ) -> (Vec<Event>, Self) {
        if !enabled {
            return (base, Self::default());
        }
        let considered = pending.iter().map(|event| event.id).collect();
        let mut selected = valid_wrappers(pending);
        let selected_ids: HashSet<_> = selected.iter().map(|event| event.id).collect();
        let (mut events, mut fresh) = partition_base(base);
        fresh.retain(|event| !selected_ids.contains(&event.id));
        fresh.sort_by(newest_first);
        let available = MAX_REPOSTS_PER_ATTEMPT.saturating_sub(selected.len());
        let deferred = fresh.split_off(fresh.len().min(available));
        selected.extend(fresh);
        let base_ids = events.iter().map(|event| event.id).collect();
        events.extend(selected.iter().cloned());
        let settlement = Self {
            enabled: true,
            base_ids,
            considered,
            selected,
            deferred,
            ..Self::default()
        };
        (events, settlement)
    }

    pub(super) fn settle_targets(
        &mut self,
        mut events: Vec<Event>,
        retry: BTreeSet<EventId>,
    ) -> Vec<Event> {
        if !self.enabled {
            return events;
        }
        self.retry = retry;
        self.support = ResolvedRepostSupport::new(&events, &self.selected);
        let present = verified_event_ids(&events);
        for index in 0..self.selected.len() {
            let status = self.target_status(index, &present);
            self.settle_wrapper_target(status);
        }
        events.retain(|event| self.keeps_target_event(event));
        events
    }

    fn target_status(&self, index: usize, present: &HashSet<EventId>) -> TargetStatus {
        let wrapper = &self.selected[index];
        let exact_target = exact_target_id(wrapper);
        TargetStatus {
            id: wrapper.id,
            materialized: self.support.materialized(&wrapper.id),
            exact: exact_target.is_some(),
            exact_present: exact_target.is_some_and(|id| present.contains(&id)),
            needs_lookup: lookup_for_enrichment(wrapper).is_some(),
        }
    }

    fn settle_wrapper_target(&mut self, status: TargetStatus) {
        if status.materialized {
            self.settle_materialized_target(status);
        } else if status.exact_present {
            self.retry.remove(&status.id);
        } else if status.needs_lookup {
            self.retry.insert(status.id);
        }
    }

    fn settle_materialized_target(&mut self, status: TargetStatus) {
        if status.exact {
            self.retry.remove(&status.id);
        }
        if !self.retry.contains(&status.id) {
            self.target_settled.insert(status.id);
        }
    }

    fn keeps_target_event(&self, event: &Event) -> bool {
        !is_wrapper(event) || self.target_settled.contains(&event.id)
    }

    pub(super) fn finish(
        mut self,
        events: Vec<Event>,
        deletion_settled: BTreeSet<EventId>,
    ) -> (Vec<Event>, RepostRetryDelta) {
        if !self.enabled {
            return (events, RepostRetryDelta::default());
        }
        self.retry.extend(
            self.target_settled
                .iter()
                .filter(|id| !deletion_settled.contains(id)),
        );
        let safe: BTreeSet<_> = self
            .target_settled
            .intersection(&deletion_settled)
            .copied()
            .collect();
        let donors = self.support.donors_for(&safe);
        self.deferred.extend(
            self.selected
                .iter()
                .filter(|event| self.retry.contains(&event.id))
                .cloned(),
        );
        let events = events
            .into_iter()
            .filter(|event| self.allowed(event, &safe, &donors))
            .collect();
        let delta = RepostRetryDelta {
            considered: self.considered,
            deferred: unique(self.deferred),
        };
        (events, delta)
    }

    fn allowed(&self, event: &Event, safe: &BTreeSet<EventId>, donors: &BTreeSet<EventId>) -> bool {
        self.base_ids.contains(&event.id)
            || event.kind == Kind::EventDeletion
            || safe.contains(&event.id)
            || donors.contains(&event.id)
    }
}

fn partition_base(events: Vec<Event>) -> (Vec<Event>, Vec<Event>) {
    let mut base = Vec::new();
    let mut wrappers = Vec::new();
    for event in events {
        if valid_wrapper(&event) {
            wrappers.push(event);
        } else if !is_wrapper(&event) {
            base.push(event);
        }
    }
    (base, wrappers)
}

fn valid_wrappers(events: Vec<Event>) -> Vec<Event> {
    unique(events.into_iter().filter(valid_wrapper))
}

fn valid_wrapper(event: &Event) -> bool {
    is_wrapper(event)
        && (feed_post_from_event(event).is_some() || reference_for_repost(event).is_some())
}

fn unique(events: impl IntoIterator<Item = Event>) -> Vec<Event> {
    let mut ids = HashSet::new();
    events
        .into_iter()
        .filter(|event| ids.insert(event.id))
        .collect()
}

fn newest_first(left: &Event, right: &Event) -> std::cmp::Ordering {
    right
        .created_at
        .cmp(&left.created_at)
        .then_with(|| left.id.cmp(&right.id))
}

fn is_wrapper(event: &Event) -> bool {
    matches!(event.kind.as_u16(), 6 | 16)
}
