use super::allocation_evidence::candidate_utility;
use super::ranges::playable_gain;
use super::resources::storage_displaces_speculation;
use super::{
    AllocationReason, CandidateSnapshot, InFlightAction, OriginHealth, PlayabilitySnapshot,
    PlayableRange, RetainedAllocation,
};
use crate::PostId;
use std::collections::HashSet;

pub(super) fn retained(
    snapshot: &PlayabilitySnapshot,
    playback_endangered: bool,
    critical_slots: usize,
    admitted: &HashSet<PostId>,
) -> Vec<RetainedAllocation> {
    let mut work = current_commitments(snapshot);
    if storage_displaces_speculation(snapshot) {
        return work;
    }
    let future = future_commitments(snapshot, admitted);
    let limit = future_limit(snapshot, playback_endangered, critical_slots, work.len());
    work.extend(future.into_iter().take(limit));
    work
}

fn current_commitments(snapshot: &PlayabilitySnapshot) -> Vec<RetainedAllocation> {
    snapshot
        .candidates
        .iter()
        .filter(|candidate| candidate.post == snapshot.playback.current)
        .flat_map(|candidate| candidate_commitments(snapshot, candidate))
        .collect()
}

fn future_commitments(
    snapshot: &PlayabilitySnapshot,
    admitted: &HashSet<PostId>,
) -> Vec<RetainedAllocation> {
    let mut candidates: Vec<_> = snapshot
        .candidates
        .iter()
        .filter(|candidate| {
            candidate.retrieval_eligible
                && candidate.post != snapshot.playback.current
                && candidate.view_probability.value() >= 0.05
                && (candidate.feed_offset.value() > 0 || admitted.contains(&candidate.post))
        })
        .collect();
    candidates.sort_by(|left, right| {
        right
            .view_probability
            .value()
            .total_cmp(&left.view_probability.value())
    });
    candidates
        .into_iter()
        .flat_map(|candidate| candidate_commitments(snapshot, candidate))
        .collect()
}

fn candidate_commitments(
    snapshot: &PlayabilitySnapshot,
    candidate: &super::CandidateSnapshot,
) -> Vec<RetainedAllocation> {
    let mut commitments: Vec<_> = candidate
        .in_flight
        .iter()
        .filter(|active| !active.cancelling)
        .filter_map(|active| retained_allocation(snapshot, candidate, active))
        .collect();
    commitments.sort_by_key(|work| {
        let bytes = work.request.requested_bytes();
        (work.committed_until_ms, bytes.start, bytes.end)
    });
    commitments
}

fn retained_allocation(
    snapshot: &PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
    active: &InFlightAction,
) -> Option<RetainedAllocation> {
    let current = candidate.post == snapshot.playback.current;
    let origin = useful_origin(snapshot, candidate, active, current)?;
    let gain = playable_gain(candidate, active.effective_bytes);
    Some(RetainedAllocation {
        action_id: active.action_id,
        post: candidate.post.clone(),
        request: active.request,
        source: active.source.clone(),
        utility: candidate_utility(
            snapshot,
            candidate,
            origin,
            PlayableRange {
                bytes: active.effective_bytes,
                playable_ms: gain,
            },
        ),
        committed_until_ms: active.committed_until_ms,
        reason: AllocationReason::UsefulCommitment,
    })
}

fn future_limit(
    snapshot: &PlayabilitySnapshot,
    endangered: bool,
    critical_slots: usize,
    current_slots: usize,
) -> usize {
    if !endangered {
        return usize::MAX;
    }
    snapshot
        .network
        .connection_ceiling
        .saturating_sub(critical_slots.saturating_add(current_slots))
}

fn useful_origin<'a>(
    snapshot: &PlayabilitySnapshot,
    candidate: &'a CandidateSnapshot,
    active: &InFlightAction,
    current: bool,
) -> Option<&'a OriginHealth> {
    let expired = active.committed_until_ms <= snapshot.observed_at_ms;
    if !active.identity_current || (expired && !current) {
        return None;
    }
    candidate
        .origins
        .iter()
        .find(|origin| origin.source == active.source && origin.available)
}
