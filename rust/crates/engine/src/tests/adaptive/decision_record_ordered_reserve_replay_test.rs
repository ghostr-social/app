#[path = "decision_record_ordered_reserve_replay_support.rs"]
mod support;

use crate::adaptive::{AdaptivePlayabilityPolicy, DecisionReplayStatus};
use support::{capture, gap_state, historical_plan, legacy_record, no_gap_state, with_schema};

#[test]
fn historical_gap_state_replays_with_its_aggregate_reserve_policy() {
    let state = gap_state();
    let historical = historical_plan(&state);
    let legacy = legacy_record(&state, &historical, 1);

    assert_eq!(legacy.integrity_status(), DecisionReplayStatus::Verified);
}

#[test]
fn historical_capability_gap_replays_with_aggregate_reserve_policy() {
    let mut state = gap_state();
    state.candidates[0].direct_playback_blocked = true;
    let historical = historical_plan(&state);
    let legacy = legacy_record(&state, &historical, 4);

    assert_eq!(legacy.integrity_status(), DecisionReplayStatus::Verified);
}

#[test]
fn current_ordered_records_use_schema_five_and_replay() {
    for state in [gap_state(), no_gap_state()] {
        let plan = AdaptivePlayabilityPolicy.plan(&state);
        let record = capture(&state, &plan);

        assert_eq!(record.schema_version, 5);
        assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
    }
}

#[test]
fn relabeling_schema_five_as_a_legacy_schema_is_rejected() {
    for state in [gap_state(), no_gap_state()] {
        let plan = AdaptivePlayabilityPolicy.plan(&state);
        let current = capture(&state, &plan);
        assert_eq!(current.schema_version, 5);
        for schema in [1, 4] {
            assert_eq!(
                with_schema(&current, schema).integrity_status(),
                DecisionReplayStatus::PlanMismatch
            );
        }
    }
}
