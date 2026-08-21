use super::super::super::DecisionRecord;
use crate::adaptive::decision::advanced::verify_search_replay;
use crate::adaptive::DecisionReplayStatus;

pub(super) fn verify(record: &DecisionRecord) -> Result<(), DecisionReplayStatus> {
    let decision = record
        .warp_decision
        .as_ref()
        .ok_or(DecisionReplayStatus::UnsupportedSchema)?;
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
