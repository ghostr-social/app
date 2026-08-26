use super::EvictionCandidate;
use crate::adaptive::Eviction;

pub(super) fn select(
    candidates: Vec<EvictionCandidate>,
    wanted: u64,
    hard: u64,
    headroom: u64,
) -> Vec<Eviction> {
    let (protected, ordinary): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|candidate| candidate.protected);
    let mut selection = Selection::new(wanted, headroom);
    selection.take(ordinary);
    if selection.released() < hard {
        selection.remaining = hard - selection.released();
        selection.take(protected);
    }
    selection.selected
}

struct Selection {
    remaining: u64,
    headroom: u64,
    selected: Vec<Eviction>,
}

impl Selection {
    fn new(remaining: u64, headroom: u64) -> Self {
        Self {
            remaining,
            headroom,
            selected: Vec::new(),
        }
    }

    fn take(&mut self, candidates: Vec<EvictionCandidate>) {
        for candidate in candidates {
            if self.remaining == 0 {
                return;
            }
            self.consider(&candidate);
        }
    }

    fn consider(&mut self, candidate: &EvictionCandidate) {
        let eviction = selected_extent(candidate, self.remaining);
        if !feasible(candidate, &eviction, self.headroom) {
            return;
        }
        let released = eviction.range.len();
        self.remaining = self.remaining.saturating_sub(released);
        self.headroom = self.headroom.saturating_add(released);
        self.selected.push(eviction);
    }

    fn released(&self) -> u64 {
        self.selected.iter().map(|item| item.range.len()).sum()
    }
}

fn feasible(candidate: &EvictionCandidate, eviction: &Eviction, headroom: u64) -> bool {
    if candidate.indivisible || candidate.physical_tail {
        return true;
    }
    candidate.present_bytes.saturating_sub(eviction.range.len()) <= headroom
}

fn selected_extent(candidate: &EvictionCandidate, maximum: u64) -> Eviction {
    if candidate.indivisible {
        return candidate.eviction.clone();
    }
    exact_tail(candidate.eviction.clone(), maximum)
}

fn exact_tail(mut eviction: Eviction, maximum: u64) -> Eviction {
    let original = eviction.range.len();
    if original <= maximum {
        return eviction;
    }
    eviction.range.start = eviction.range.end - maximum;
    eviction.expected_playable_loss_ms *= maximum as f64 / original as f64;
    eviction
}
