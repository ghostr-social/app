use super::super::super::DecisionRecord;
use crate::adaptive::decision::advanced::verify_search_reserve;
use crate::adaptive::{DecisionReplayStatus, RecordedWarpDecision};

pub(super) fn verify(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    let Some(input) = decision.search_replay_input.as_ref() else {
        return Ok(());
    };
    verify_search_reserve(input, &decision.reserve)?;
    let deadline = record.replay_state.snapshot().commitment_ms;
    require(
        decision
            .reserve
            .chance
            .is_none_or(|chance| chance.deadline_ms == deadline),
    )
}

fn require(value: bool) -> Result<(), DecisionReplayStatus> {
    value
        .then_some(())
        .ok_or(DecisionReplayStatus::PlanMismatch)
}
