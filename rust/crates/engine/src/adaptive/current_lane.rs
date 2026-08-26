use super::allocation::{append_candidate, AppendInputs};
use super::plan::AllocationPlan;
use super::ranges::{missing, playable_gain};
use super::reserves::{planned_bytes, planned_gain};
use super::{CandidateSnapshot, CurrentAuthority, PlayabilitySnapshot};

const EMERGENCY_DEPTH_MS: u64 = 12_000;
const SAFE_CURRENT_DEPTH_MS: u64 = 6_000;

pub(super) struct CurrentLane<'a> {
    candidate: &'a CandidateSnapshot,
    pub(super) emergency: bool,
    pub(super) storage_room: u64,
    target_ms: u64,
    authority: CurrentAuthority,
}

impl<'a> CurrentLane<'a> {
    pub(super) fn new(
        snapshot: &PlayabilitySnapshot,
        candidate: &'a CandidateSnapshot,
        emergency: bool,
        plan: &AllocationPlan,
    ) -> Self {
        Self {
            candidate,
            emergency,
            storage_room: storage_room(snapshot, plan),
            target_ms: target_ms(snapshot, candidate, emergency),
            authority: snapshot.playback.authority,
        }
    }

    pub(super) fn append_start(&self, plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot) {
        if self.target_ms == 0 || !self.start_allowed() {
            return;
        }
        append_candidate(plan, snapshot, self.inputs(1, self.storage_room));
    }

    pub(super) fn append_depth(&self, plan: &mut AllocationPlan, snapshot: &PlayabilitySnapshot) {
        if self.authority == CurrentAuthority::Provisional {
            return;
        }
        let target = self
            .target_ms
            .saturating_sub(planned_gain(plan, self.candidate));
        let budget = self.storage_room.saturating_sub(planned_bytes(plan));
        append_candidate(plan, snapshot, self.inputs(target, budget));
    }

    pub(super) fn protected(&self, plan: &AllocationPlan) -> bool {
        missing(self.candidate).is_empty()
            || plan
                .allocations
                .iter()
                .any(|work| work.post == self.candidate.post)
            || plan
                .retained
                .iter()
                .any(|work| work.post == self.candidate.post)
    }

    fn inputs(&self, target_ms: u64, budget: u64) -> AppendInputs<'a> {
        let reason = self
            .candidate
            .needs_bootstrap()
            .then_some(super::AllocationReason::MediaBootstrap);
        AppendInputs {
            candidate: self.candidate,
            target_ms,
            emergency: self.emergency,
            budget,
            reason,
        }
    }

    fn start_allowed(&self) -> bool {
        self.authority == CurrentAuthority::Canonical || self.candidate.needs_bootstrap()
    }
}

pub(super) fn inflight_need_bytes(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .in_flight
        .iter()
        .filter(|active| active.identity_current)
        .map(|active| active.reserved_storage_bytes)
        .sum()
}

fn target_ms(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    emergency: bool,
) -> u64 {
    let depth = if emergency {
        EMERGENCY_DEPTH_MS
    } else {
        SAFE_CURRENT_DEPTH_MS
    };
    depth.saturating_sub(
        snapshot
            .playback
            .buffer_ahead_ms
            .saturating_add(inflight_playable_ms(candidate)),
    )
}

fn inflight_playable_ms(candidate: &CandidateSnapshot) -> u64 {
    candidate
        .in_flight
        .iter()
        .filter(|active| active.identity_current)
        .filter(|active| !active.cancelling)
        .map(|active| playable_gain(candidate, active.effective_bytes))
        .sum()
}

fn storage_room(snapshot: &PlayabilitySnapshot, plan: &AllocationPlan) -> u64 {
    let released: u64 = plan.evictions.iter().map(|item| item.range.len()).sum();
    snapshot.storage.available_bytes().saturating_add(released)
}
