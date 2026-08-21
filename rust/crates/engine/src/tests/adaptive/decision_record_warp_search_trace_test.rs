use super::super::decision_record_warp_test_support::{add_generated_action, decision, record};
use crate::adaptive::{
    ActionKind, PlannerCommand, RecordedSearchPruneReason, RetainedSearchPlan, SearchPruneReason,
};
use crate::PostId;

#[test]
fn schema_two_search_trace_keeps_distinct_private_action_definitions() {
    let captured = record(&search_trace());
    let advanced = captured.warp_decision.unwrap();
    assert_trace(&advanced);
    assert_private(&advanced);
}

fn search_trace() -> crate::adaptive::WarpPlanningDecision {
    let mut value = decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://selected.example/media?cap=selected".into(),
        },
        ActionKind::Head,
    );
    add_generated_action(&mut value, 8, "https://unselected.example/media?cap=other");
    add_generated_action(
        &mut value,
        9,
        "https://filtered.example/media?cap=discarded",
    );
    value.admissible_action_ids = vec![7, 8];
    value.search.retained_plans = vec![RetainedSearchPlan {
        action_ids: vec![7],
        score_micros: 8_000,
    }];
    value.search.pruned_plans = vec![crate::adaptive::PrunedSearchPlan {
        action_ids: vec![8],
        reason: SearchPruneReason::BeamWidth,
    }];
    value.search.pruned_plan_events_total = 1;
    value.pruned_action_ids = vec![9];
    value
}

fn assert_trace(advanced: &crate::adaptive::RecordedWarpDecision) {
    assert_eq!(ids(&advanced.admissible_actions), vec![7, 8]);
    assert_eq!(advanced.admissible_actions_total, 2);
    assert_eq!(
        ids(&advanced.unattributed_pre_search_pruned_actions),
        vec![9]
    );
    assert_eq!(advanced.unattributed_pre_search_pruned_actions_total, 1);
    assert_eq!(ids(&advanced.search.retained_plans[0].actions), vec![7]);
    assert_eq!(advanced.search.retained_plans_total, 1);
    assert_eq!(
        ids(&advanced.search.chosen_plan.as_ref().unwrap().actions),
        vec![7]
    );
    let pruned = &advanced.search.recorded_pruned_plans[0];
    assert_eq!(ids(&pruned.actions), vec![8]);
    assert_eq!(pruned.reason, RecordedSearchPruneReason::BeamWidth);
    assert_eq!(advanced.search.pruned_plan_events_total, 1);
    assert!(!advanced.search.pruned_plan_sample_truncated);
    assert_eq!(advanced.search.committed_actions, 1);
    assert!(!advanced.search.used_greedy_fallback);
    assert_eq!(advanced.search.common_random_seed, 99);
    assert_eq!(
        serde_json::to_value(pruned).unwrap()["reason"],
        serde_json::json!("beam_width")
    );
}

fn assert_private(advanced: &crate::adaptive::RecordedWarpDecision) {
    let serialized = serde_json::to_string(&advanced).unwrap();
    for private in ["selected.example", "unselected.example", "filtered.example"] {
        assert!(!serialized.contains(private));
    }
}

fn ids(actions: &[crate::adaptive::RecordedWarpAction]) -> Vec<u16> {
    actions
        .iter()
        .map(|action| action.planner_action_id)
        .collect()
}
