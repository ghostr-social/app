use super::super::super::DecisionRecord;
use crate::adaptive::decision::advanced::{verify_planner_replay, verify_search_replay};
use crate::adaptive::DecisionReplayStatus;

pub(super) fn verify(record: &DecisionRecord) -> Result<(), DecisionReplayStatus> {
    let decision = record
        .warp_decision
        .as_ref()
        .ok_or(DecisionReplayStatus::UnsupportedSchema)?;
    if let Some(capsule) = &decision.planner_replay_capsule {
        return verify_planner_replay(capsule, &record.replay_state.snapshot(), decision);
    }
    let input = decision
        .search_replay_input
        .as_ref()
        .ok_or(DecisionReplayStatus::AdvancedReplayUnavailable)?;
    let selected = decision
        .selected
        .as_ref()
        .map(|action| action.planner_action_id);
    verify_search_replay(input, &decision.search, selected, &decision.reserve)
}
