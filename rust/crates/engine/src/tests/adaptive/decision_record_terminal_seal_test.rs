use crate::adaptive::{
    ActionKind, DecisionOutcome, DecisionRecord, DecisionReplayStatus, PlannerCommand,
    ResourceCost, TransformKind,
};
use crate::tests::adaptive::decision_record_warp_test_support::{decision, record};
use crate::{ActionId, PostId};

#[test]
fn schema_three_requires_a_seal_for_initial_pending_and_noop_records() {
    let pending = selected_record();
    assert_sealed(&pending);

    let mut noop = selected_decision();
    noop.selected = None;
    noop.evaluation = None;
    noop.search = Default::default();
    noop.admissible_action_ids.clear();
    noop.pruned_action_ids = vec![7];
    assert_sealed(&record(&noop));
}

#[test]
fn removing_or_downgrading_terminal_evidence_never_restores_verified() {
    let mut record = selected_record();
    assert!(record.bind_action(ActionId::new(7)));
    assert!(record.resolve_with_resources(
        DecisionOutcome::Succeeded {
            bytes: 32,
            elapsed_ms: 4,
        },
        ResourceCost::new(0, 32, 7, 0),
    ));
    let terminal = serde_json::to_value(record).unwrap();

    let mut missing = terminal.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("terminal_evidence_hash");
    assert_mismatch(missing);

    let mut reverted = terminal;
    reverted["eventual_outcome"] = serde_json::json!({ "status": "pending" });
    reverted["actual_resources"] = serde_json::Value::Null;
    reverted
        .as_object_mut()
        .unwrap()
        .remove("terminal_evidence_hash");
    assert_mismatch(reverted);
}

#[test]
fn schema_downgrade_cannot_turn_new_records_into_legacy_payloads() {
    let original = serde_json::to_value(selected_record()).unwrap();
    let mut v2 = original.clone();
    v2["schema_version"] = serde_json::json!(2);
    v2.as_object_mut().unwrap().remove("terminal_evidence_hash");
    assert_mismatch(v2);

    let mut v1 = original;
    v1["schema_version"] = serde_json::json!(1);
    let v1: DecisionRecord = serde_json::from_value(v1).unwrap();
    assert_eq!(v1.replay(), DecisionReplayStatus::UnsupportedSchema);
}

fn assert_sealed(record: &DecisionRecord) {
    assert_eq!(record.schema_version, 3);
    let value = serde_json::to_value(record).unwrap();
    assert!(value["terminal_evidence_hash"].is_string());
    assert_eq!(record.replay(), DecisionReplayStatus::Verified);
}

fn assert_mismatch(value: serde_json::Value) {
    let record: DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(record.replay(), DecisionReplayStatus::PlanMismatch);
}

fn selected_record() -> DecisionRecord {
    record(&selected_decision())
}

fn selected_decision() -> crate::adaptive::WarpPlanningDecision {
    decision(
        "secret-post",
        PlannerCommand::Transform {
            post: PostId::new("secret-post"),
            kind: TransformKind::Remux,
        },
        ActionKind::Transform(TransformKind::Remux),
    )
}
