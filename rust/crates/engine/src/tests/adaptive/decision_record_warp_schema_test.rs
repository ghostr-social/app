#[path = "decision_record_warp_search_trace_test.rs"]
mod search_trace_test;

use super::decision_record_warp_test_support::{decision, record};
use crate::adaptive::{
    ActionKind, DecisionOutcome, DecisionRecord, DecisionReplayStatus, PlannerCommand,
    RecordedWarpCommand, WarpPlanningDecision,
};
use crate::PostId;

#[test]
fn schema_two_records_the_authoritative_warp_selection_and_noop() {
    let source = "https://private.example/media.mp4?cap=raw";
    let selected = decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: source.into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    );
    let captured = record(&selected);
    assert_selected(&captured);
    assert_noop(&record(&noop(selected)));
}

fn assert_selected(captured: &DecisionRecord) {
    let action = captured
        .warp_decision
        .as_ref()
        .unwrap()
        .selected
        .as_ref()
        .unwrap();

    assert_eq!(captured.schema_version, 2);
    assert_eq!(action.planner_action_id, 7);
    assert!(matches!(
        action.command,
        RecordedWarpCommand::ProbeHead { .. }
    ));
    assert_eq!(captured.chosen_action.as_ref().unwrap().request, "head");
    assert_eq!(captured.eventual_outcome, DecisionOutcome::Pending);
    assert!(captured.retained_plans.is_empty());
    assert!(captured.pruned.is_empty());
    assert_eq!(captured.shadow_prices.storage_time_micros, 20);
    assert_eq!(captured.random_seed, 99);
    assert_eq!(
        captured.replay(),
        DecisionReplayStatus::AdvancedReplayUnavailable
    );
}

fn noop(mut noop: WarpPlanningDecision) -> WarpPlanningDecision {
    noop.selected = None;
    noop.evaluation = None;
    noop.search = Default::default();
    noop.admissible_action_ids.clear();
    noop.pruned_action_ids = vec![7];
    noop
}

fn assert_noop(noop: &DecisionRecord) {
    assert_eq!(noop.random_seed, 99);
    assert!(noop.warp_decision.as_ref().unwrap().selected.is_none());
    assert_eq!(
        noop.eventual_outcome,
        DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0
        }
    )
}
