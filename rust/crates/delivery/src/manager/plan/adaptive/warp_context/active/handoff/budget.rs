use super::Input;
use crate::manager::inflight::ActiveAction;
use crate::manager::plan::PlanInputs;
use ghostr_engine::adaptive::{Allocation, PreemptionAuthority};
use ghostr_engine::{ActionId, RequestAuthority};
use std::collections::{BTreeMap, HashSet};

pub(super) struct RequestBudget {
    global_limit: usize,
    authority_limit: usize,
    global_used: usize,
    authorities: BTreeMap<RequestAuthority, usize>,
}

impl RequestBudget {
    pub(super) fn seeded(input: Input<'_>, scoped: &HashSet<ActionId>) -> Self {
        let mut budget = Self::new(input.inputs);
        seed_existing(&mut budget, input.inputs, scoped);
        if let Some(source) = missing_critical_source(input) {
            budget.occupy(source);
        }
        budget
    }

    fn new(inputs: &PlanInputs<'_>) -> Self {
        Self {
            global_limit: inputs.connection_ceiling.max(1),
            authority_limit: inputs.per_authority_request_limit.max(1),
            global_used: 0,
            authorities: BTreeMap::new(),
        }
    }

    pub(super) fn admit(&mut self, source: &str) -> bool {
        let Some(authority) = RequestAuthority::from_url(source) else {
            return false;
        };
        let used = self.authorities.get(&authority).copied().unwrap_or(0);
        if self.global_used >= self.global_limit || used >= self.authority_limit {
            return false;
        }
        self.occupy_authority(authority);
        true
    }

    fn occupy(&mut self, source: &str) {
        self.global_used = self.global_used.saturating_add(1);
        if let Some(authority) = RequestAuthority::from_url(source) {
            let used = self.authorities.entry(authority).or_default();
            *used = used.saturating_add(1);
        }
    }

    fn occupy_authority(&mut self, authority: RequestAuthority) {
        self.global_used = self.global_used.saturating_add(1);
        let used = self.authorities.entry(authority).or_default();
        *used = used.saturating_add(1);
    }
}

fn seed_existing(budget: &mut RequestBudget, inputs: &PlanInputs<'_>, scoped: &HashSet<ActionId>) {
    inputs
        .in_flight
        .iter()
        .filter(|active| {
            !active.io_finished() && !active.cancelling() && !scoped.contains(&active.action_id())
        })
        .for_each(|active| budget.occupy(active.identity().source().as_str()));
    inputs
        .active_head_probes
        .iter()
        .for_each(|identity| budget.occupy(identity.source().as_str()));
    inputs
        .active_hls_sources
        .iter()
        .for_each(|source| budget.occupy(source));
}

fn missing_critical_source<'a>(input: Input<'a>) -> Option<&'a str> {
    input
        .base
        .allocations
        .iter()
        .filter(|allocation| allocation.authority == PreemptionAuthority::PlaybackCritical)
        .find(|allocation| {
            !input
                .inputs
                .in_flight
                .iter()
                .any(|active| action_matches(input, allocation, active))
        })
        .map(|allocation| allocation.source.as_str())
        .or_else(|| pending_current_hls_source(input))
}

fn pending_current_hls_source(input: Input<'_>) -> Option<&str> {
    let current = &input.snapshot.playback.current;
    if input.inputs.retry.cooling_until(current).is_some() {
        return None;
    }
    input
        .snapshot
        .hls_candidates
        .iter()
        .find(|candidate| &candidate.post == current)
        .and_then(|candidate| {
            candidate.pending_request_source(
                input.snapshot.request_slice_bytes,
                input.inputs.segmented_storage_available_bytes,
            )
        })
}

fn action_matches(input: Input<'_>, allocation: &Allocation, active: &ActiveAction) -> bool {
    let identity = input
        .state
        .catalog()
        .transfer_identity(&allocation.post, &allocation.source);
    let active_bytes = active.effective_bytes();
    let planned_bytes = allocation.request.requested_bytes();
    !active.cancelling()
        && !active.io_finished()
        && active.post() == &allocation.post
        && identity.as_ref() == Some(active.identity())
        && active_bytes.start < planned_bytes.end
        && planned_bytes.start < active_bytes.end
}
