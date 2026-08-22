use super::support::{planned, record};
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn segmented_storage_budget_is_replay_mutation_sensitive() {
    let (state, decision) = planned();
    let captured = record(&state, &decision);
    let mut value = serde_json::to_value(captured).unwrap();
    value["warp_decision"]["search_replay_input"]["budget"]["segmented_storage_bytes"] =
        32_u64.into();
    let tampered: DecisionRecord = serde_json::from_value(value).unwrap();

    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
}
