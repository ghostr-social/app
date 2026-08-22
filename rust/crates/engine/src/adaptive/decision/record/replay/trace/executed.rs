use super::super::super::DecisionRecord;
use crate::adaptive::decision::advanced::executed_coherent;
use crate::adaptive::{DecisionReplayStatus, RecordedWarpDecision};

pub(super) fn verify(
    record: &DecisionRecord,
    decision: &RecordedWarpDecision,
) -> Result<(), DecisionReplayStatus> {
    let Some(executed) = record.executed_request.as_ref() else {
        return Ok(());
    };
    let coherent = record.chosen_action_id.is_some()
        && decision
            .selected
            .as_ref()
            .is_some_and(|selected| executed_coherent(executed, selected));
    coherent
        .then_some(())
        .ok_or(DecisionReplayStatus::PlanMismatch)
}
