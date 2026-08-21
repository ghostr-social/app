use super::decision_record_warp_test_support::{decision, record};
use crate::adaptive::{
    ActionKind, DecisionOutcome, DecisionReplayStatus, PlannerCommand, RecordedWarpCommand,
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
        },
        ActionKind::Head,
    );
    let captured = record(&selected);
    let advanced = captured.warp_decision.as_ref().unwrap();
    let action = advanced.selected.as_ref().unwrap();

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
    assert_eq!(captured.shadow_prices.request_micros, 40);
    assert_eq!(advanced.prices.storage_micros, 2);
    assert_eq!(captured.random_seed, 99);
    let value = serde_json::to_value(&captured).unwrap();
    assert!(value.get("warp_decision_hash").is_none());
    assert!(value["replay_plan_hash"]
        .as_str()
        .unwrap()
        .starts_with("warp-v2-decision:"));
    assert_eq!(
        captured.replay(),
        DecisionReplayStatus::AdvancedReplayUnavailable
    );

    let mut noop = selected;
    noop.selected = None;
    noop.evaluation = None;
    let noop = record(&noop);
    assert!(noop.warp_decision.unwrap().selected.is_none());
    assert!(noop.chosen_action.is_none());
    assert_eq!(
        noop.eventual_outcome,
        DecisionOutcome::Succeeded {
            bytes: 0,
            elapsed_ms: 0
        }
    );
}
