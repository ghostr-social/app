mod action;
mod coherence;
mod completeness;
mod fresh_search;

use super::super::DecisionRecord;
use crate::adaptive::{DecisionReplayStatus, VerifiedWarpReplay};

pub(super) fn reconstruct(
    record: &DecisionRecord,
) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
    let decision = record
        .warp_decision
        .as_ref()
        .ok_or(DecisionReplayStatus::UnsupportedSchema)?;
    completeness::verify(decision)?;
    coherence::verify(record, decision)?;
    Ok(VerifiedWarpReplay::new(
        record.sequence,
        record.state_hash.clone(),
        record.replay_plan_hash.clone(),
        decision.clone(),
    ))
}

pub(super) fn verify_fresh_search(record: &DecisionRecord) -> Result<(), DecisionReplayStatus> {
    fresh_search::verify(record)
}
