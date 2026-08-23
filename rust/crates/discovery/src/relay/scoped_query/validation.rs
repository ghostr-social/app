use anyhow::bail;
use nostr_sdk::{Event, EventId, Filter};
use std::collections::HashSet;

const ABSOLUTE_QUERY_EVENT_LIMIT: usize = 1_000;
const CANDIDATE_MULTIPLIER: usize = 4;
const ABSOLUTE_QUERY_CANDIDATE_LIMIT: usize = ABSOLUTE_QUERY_EVENT_LIMIT * CANDIDATE_MULTIPLIER;

pub(super) fn structural_filter(filter: &Filter) -> Filter {
    let mut structural = filter.clone();
    structural.search = None;
    structural
}

pub(super) struct ValidationState {
    filter: Filter,
    seen: HashSet<EventId>,
    accepted_limit: usize,
    candidates: usize,
    candidate_limit: usize,
    overflowed: bool,
}

impl ValidationState {
    pub(super) fn new(filter: &Filter) -> Self {
        Self {
            filter: structural_filter(filter),
            seen: HashSet::new(),
            accepted_limit: event_limit(filter),
            candidates: 0,
            candidate_limit: candidate_limit(filter),
            overflowed: false,
        }
    }

    pub(super) fn accept(&mut self, event: &Event) -> anyhow::Result<bool> {
        self.candidates = self.candidates.saturating_add(1);
        if self.candidates > self.candidate_limit {
            bail!("relay exceeded query candidate limit");
        }
        if !self.filter.match_event(event) || self.seen.contains(&event.id) {
            return Ok(false);
        }
        if event.verify().is_err() {
            return Ok(false);
        }
        if self.seen.len() >= self.accepted_limit {
            self.overflowed = true;
            return Ok(false);
        }
        self.seen.insert(event.id);
        Ok(true)
    }

    pub(super) fn overflowed(&self) -> bool {
        self.overflowed
    }
}

pub(crate) fn event_limit(filter: &Filter) -> usize {
    filter
        .limit
        .unwrap_or(ABSOLUTE_QUERY_EVENT_LIMIT)
        .min(ABSOLUTE_QUERY_EVENT_LIMIT)
}

fn candidate_limit(filter: &Filter) -> usize {
    event_limit(filter)
        .saturating_mul(CANDIDATE_MULTIPLIER)
        .min(ABSOLUTE_QUERY_CANDIDATE_LIMIT)
}
