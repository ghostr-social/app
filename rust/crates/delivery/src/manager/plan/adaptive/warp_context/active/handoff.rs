use super::super::PlanInputs;
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{AllocationPlan, RetrievalRequest, MEDIA_BOOTSTRAP_PROBE_BYTES};
use ghostr_engine::ActionId;
use std::cmp::Reverse;
use std::collections::HashSet;

mod budget;
use budget::RequestBudget;

#[cfg(test)]
#[path = "handoff_request_test.rs"]
mod handoff_request_test;

#[derive(Clone, Copy)]
pub(super) struct Input<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) snapshot: &'a ghostr_engine::adaptive::PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) inputs: &'a PlanInputs<'a>,
}

pub(super) struct Admission {
    actions: HashSet<ActionId>,
}

struct Candidate<'a> {
    rank: usize,
    active: &'a ActiveAction,
}

impl Admission {
    pub(super) fn build(input: Input<'_>) -> Self {
        let scoped = scoped_actions(input);
        let mut candidates = candidates(input);
        candidates.sort_by_key(|candidate| {
            let active = candidate.active;
            (
                candidate.rank,
                Reverse(active.effective_bytes().start),
                active.effective_bytes().len(),
                active.launched_at_ms(),
                active.action_id(),
            )
        });
        let mut budget = RequestBudget::seeded(input, &scoped);
        let mut admitted_posts = HashSet::new();
        let actions = candidates
            .into_iter()
            .filter_map(|candidate| {
                let active = candidate.active;
                if admitted_posts.contains(active.post())
                    || !budget.admit(active.identity().source().as_str())
                {
                    return None;
                }
                admitted_posts.insert(active.post().clone());
                Some(active.action_id())
            })
            .collect();
        Self { actions }
    }

    pub(super) fn permits(&self, action: ActionId) -> bool {
        self.actions.contains(&action)
    }
}

fn candidates<'a>(input: Input<'a>) -> Vec<Candidate<'a>> {
    input
        .inputs
        .in_flight
        .iter()
        .filter_map(|active| candidate(input, active))
        .collect()
}

fn scoped_actions(input: Input<'_>) -> HashSet<ActionId> {
    input
        .inputs
        .in_flight
        .iter()
        .filter(|active| {
            input
                .state
                .provisional_handoff_rank(active.post(), active.launched_at_ms())
                .is_some()
        })
        .map(ActiveAction::action_id)
        .collect()
}

fn candidate<'a>(input: Input<'a>, active: &'a ActiveAction) -> Option<Candidate<'a>> {
    let rank = input
        .state
        .provisional_handoff_rank(active.post(), active.launched_at_ms())?;
    valid(input, active).then_some(Candidate { rank, active })
}

fn valid(input: Input<'_>, active: &ActiveAction) -> bool {
    let source = active.identity().source().as_str();
    !active.io_finished()
        && !active.cancelling()
        && active.committed_until_ms() > input.snapshot.observed_at_ms
        && exact_provisional_bootstrap(active.request())
        && input
            .state
            .catalog()
            .deliverable_transfer_identity(active.post(), source)
            .as_ref()
            == Some(active.identity())
}

fn exact_provisional_bootstrap(request: RetrievalRequest) -> bool {
    matches!(
        request,
        RetrievalRequest::FetchRange { bytes, .. }
            if bytes.start == 0 && bytes.end == MEDIA_BOOTSTRAP_PROBE_BYTES
    )
}
