use super::privacy::DecisionPrivacy;
use super::types::{DecisionAction, PrunedCandidate, PrunedReason};
use crate::adaptive::{AllocationPlan, CandidateSnapshot, NextReserveEvidence, RetrievalRequest};
use crate::PostId;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

const RECORD_LIMIT: usize = 64;

pub(super) fn actions(plan: &AllocationPlan, privacy: &DecisionPrivacy) -> Vec<DecisionAction> {
    let allocated = plan
        .allocations
        .iter()
        .map(|item| allocation(item, privacy));
    let retained = plan.retained.iter().map(|item| DecisionAction {
        post_id: privacy.post(item.post.as_str()),
        source_id: privacy.source(&item.source),
        request: request_name(item.request),
        bytes_start: item.request.requested_bytes().start,
        bytes_end: item.request.requested_bytes().end,
        expected_playable_gain_ms: item.utility.additional_playable_ms,
        utility_micros: utility(item.utility.score),
        reason: format!("{:?}", item.reason),
        retained: true,
    });
    allocated.chain(retained).take(RECORD_LIMIT).collect()
}

pub(super) fn admissible(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    privacy: &DecisionPrivacy,
) -> Vec<String> {
    snapshot
        .candidates
        .iter()
        .filter(|item| item.retrieval_eligible)
        .filter(|item| item.origins.iter().any(|origin| origin.available))
        .take(RECORD_LIMIT)
        .map(|item| privacy.post(item.post.as_str()))
        .collect()
}

pub(super) fn pruned(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    plan: &AllocationPlan,
    privacy: &DecisionPrivacy,
) -> Vec<PrunedCandidate> {
    let selected = selected_posts(plan);
    snapshot
        .candidates
        .iter()
        .filter(|item| !selected.contains(&item.post))
        .take(RECORD_LIMIT)
        .map(|item| PrunedCandidate {
            post_id: privacy.post(item.post.as_str()),
            reasons: prune_reasons(snapshot, item),
        })
        .collect()
}

pub(super) fn capture_hash(plan: &AllocationPlan, privacy: &DecisionPrivacy) -> String {
    let mut sanitized = plan.clone();
    sanitize(&mut sanitized, privacy);
    replay_hash(&sanitized)
}

pub(super) fn replay_hash(plan: &AllocationPlan) -> String {
    let mut digest = Sha256::new();
    digest.update(format!("{plan:?}").as_bytes());
    hex(&digest.finalize())
}

fn allocation(item: &crate::adaptive::Allocation, privacy: &DecisionPrivacy) -> DecisionAction {
    let bytes = item.request.requested_bytes();
    DecisionAction {
        post_id: privacy.post(item.post.as_str()),
        source_id: privacy.source(&item.source),
        request: request_name(item.request),
        bytes_start: bytes.start,
        bytes_end: bytes.end,
        expected_playable_gain_ms: item.expected_playable_gain_ms,
        utility_micros: utility(item.utility.score),
        reason: format!("{:?}", item.reason),
        retained: false,
    }
}

fn request_name(request: RetrievalRequest) -> String {
    match request {
        RetrievalRequest::FetchRange {
            promotion: Some(_), ..
        } => "promotable_range".into(),
        RetrievalRequest::FetchRange { .. } => "range".into(),
        RetrievalRequest::FetchWhole { .. } => "whole".into(),
    }
}

fn utility(value: f64) -> i64 {
    if !value.is_finite() {
        return 0;
    }
    (value * 1_000_000.0).clamp(i64::MIN as f64, i64::MAX as f64) as i64
}

fn selected_posts(plan: &AllocationPlan) -> HashSet<PostId> {
    plan.allocations
        .iter()
        .map(|item| item.post.clone())
        .chain(plan.retained.iter().map(|item| item.post.clone()))
        .collect()
}

fn prune_reasons(
    snapshot: &crate::adaptive::PlayabilitySnapshot,
    candidate: &CandidateSnapshot,
) -> Vec<PrunedReason> {
    let mut reasons = Vec::new();
    if !candidate.retrieval_eligible {
        reasons.push(PrunedReason::RetrievalIneligible);
    }
    if !candidate.origins.iter().any(|origin| origin.available) {
        reasons.push(PrunedReason::NoAvailableOrigin);
    }
    if candidate.finalized || startup_present(candidate) {
        reasons.push(PrunedReason::AlreadyReady);
    }
    if snapshot.storage.available_bytes() == 0 {
        reasons.push(PrunedReason::NoStorageCapacity);
    }
    if snapshot.network.connection_capacity == 0 {
        reasons.push(PrunedReason::NoTransferCapacity);
    }
    if reasons.is_empty() {
        reasons.push(PrunedReason::LowerUtility);
    }
    reasons
}

fn startup_present(candidate: &CandidateSnapshot) -> bool {
    candidate.startup.as_ref().is_some_and(|startup| {
        startup.ranges().iter().all(|needed| {
            candidate
                .present
                .iter()
                .any(|have| have.start <= needed.start && needed.end <= have.end)
        })
    })
}

fn sanitize(plan: &mut AllocationPlan, privacy: &DecisionPrivacy) {
    for item in &mut plan.allocations {
        item.post = PostId::new(privacy.post(item.post.as_str()));
        item.source = privacy.source(&item.source);
    }
    for item in &mut plan.retained {
        item.post = PostId::new(privacy.post(item.post.as_str()));
        item.source = privacy.source(&item.source);
    }
    for item in &mut plan.evictions {
        item.post = PostId::new(privacy.post(item.post.as_str()));
    }
    for item in &mut plan.ready_reserve.candidates {
        item.post = PostId::new(privacy.post(item.post.as_str()));
    }
    sanitize_next(&mut plan.next_reserve, privacy);
}

fn sanitize_next(value: &mut NextReserveEvidence, privacy: &DecisionPrivacy) {
    let post = match value {
        NextReserveEvidence::Ready { post, .. }
        | NextReserveEvidence::Structural { post, .. }
        | NextReserveEvidence::InFlight { post }
        | NextReserveEvidence::Granted { post, .. }
        | NextReserveEvidence::Infeasible { post, .. } => Some(post),
        NextReserveEvidence::NotApplicable => None,
    };
    if let Some(post) = post {
        *post = PostId::new(privacy.post(post.as_str()));
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
