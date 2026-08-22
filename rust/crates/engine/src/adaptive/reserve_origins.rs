use super::sources::best_origin;
use super::CandidateSnapshot;
use crate::RequestAuthority;
use std::collections::HashMap;

pub(super) struct OriginSlots {
    counts: HashMap<RequestAuthority, usize>,
    limit: Option<usize>,
}

impl OriginSlots {
    pub(super) fn new(candidates: &[&CandidateSnapshot], limit: Option<usize>) -> Self {
        let mut slots = Self {
            counts: HashMap::new(),
            limit,
        };
        for candidate in candidates {
            if has_live_body(candidate) {
                slots.occupy(candidate);
            }
        }
        slots
    }

    pub(super) fn available(&self, candidate: &CandidateSnapshot) -> bool {
        let Some(key) = origin(candidate) else {
            return true;
        };
        let Some(limit) = self.limit else { return true };
        self.counts.get(&key).copied().unwrap_or(0) < limit
    }

    pub(super) fn occupy(&mut self, candidate: &CandidateSnapshot) {
        if let Some(key) = origin(candidate) {
            *self.counts.entry(key).or_default() += 1;
        }
    }
}

fn has_live_body(candidate: &CandidateSnapshot) -> bool {
    candidate
        .in_flight
        .iter()
        .any(|active| active.identity_current)
}

fn origin(candidate: &CandidateSnapshot) -> Option<RequestAuthority> {
    let source = &best_origin(candidate)?.source;
    RequestAuthority::from_url(source)
}
