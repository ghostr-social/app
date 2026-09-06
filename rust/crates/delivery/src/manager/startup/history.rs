use super::{DeliveryState, StartupCertificate};
use ghostr_engine::adaptive::{CandidateSnapshot, PlayabilitySnapshot};
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::collections::HashMap;

pub(super) fn certificates(
    state: &DeliveryState,
    plan: Option<&PlayabilitySnapshot>,
    stored: &HashMap<PostId, StoredMediaSnapshot>,
) -> Vec<StartupCertificate> {
    let mut candidates: Vec<_> = plan
        .into_iter()
        .flat_map(|snapshot| &snapshot.candidates)
        .filter(|candidate| eligible(candidate))
        .collect();
    candidates.sort_by_key(|candidate| candidate.feed_offset.value().unsigned_abs());
    candidates
        .into_iter()
        .filter_map(|candidate| {
            super::issue(state, &candidate.post, candidate.startup.as_ref()?, stored)
        })
        .collect()
}

fn eligible(candidate: &CandidateSnapshot) -> bool {
    candidate.feed_offset.value() < 0
        && candidate.retrieval_eligible
        && !candidate.direct_playback_blocked
}
