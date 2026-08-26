use crate::adaptive::{
    ActionKind, DecisionOutcome, DecisionRecord, DecisionReplayStatus, PlannerCommand,
    RecordedResourceCost, ResourceCost, TransformKind,
};
use crate::tests::adaptive::decision_record_warp_test_support::{decision, record};
use crate::{ActionId, PostId};

#[test]
fn terminal_transform_resources_are_atomic_and_privacy_safe() {
    let command = PlannerCommand::Transform {
        post: PostId::new("secret-post"),
        kind: TransformKind::Remux,
    };
    let mut record = record(&decision(
        "secret-post",
        command,
        ActionKind::Transform(TransformKind::Remux),
    ));
    let actual = ResourceCost::new(0, 32, 7, 0);
    let succeeded = DecisionOutcome::Succeeded {
        bytes: 32,
        elapsed_ms: 11,
    };

    assert!(record.bind_action(ActionId::new(7)));
    assert!(!record.resolve_with_resources(DecisionOutcome::Pending, actual));
    assert_eq!(record.eventual_outcome, DecisionOutcome::Pending);
    assert_eq!(record.actual_resources, None);
    assert!(record.resolve_with_resources(succeeded.clone(), actual));
    assert_eq!(
        record.actual_resources,
        Some(RecordedResourceCost::from(actual))
    );
    assert!(!record.resolve_with_resources(DecisionOutcome::Superseded, ResourceCost::default()));
    assert_eq!(record.eventual_outcome, succeeded);
    assert_eq!(
        record.actual_resources,
        Some(RecordedResourceCost::from(actual))
    );
    let json = serde_json::to_string(&record).expect("valid test fixture");
    assert!(json.contains("\"cpu_ms\":7"));
    assert!(json.contains("terminal_evidence_hash"));
    assert!(!json.contains("secret-post"));

    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
    let mut tampered = serde_json::to_value(&record).expect("valid test fixture");
    tampered["actual_resources"]["cpu_ms"] = serde_json::json!(8);
    let tampered: DecisionRecord = serde_json::from_value(tampered).expect("valid test fixture");
    assert_eq!(
        tampered.integrity_status(),
        DecisionReplayStatus::PlanMismatch
    );

    let mut tampered = serde_json::to_value(&record).expect("valid test fixture");
    tampered["eventual_outcome"]["elapsed_ms"] = serde_json::json!(12);
    let tampered: DecisionRecord = serde_json::from_value(tampered).expect("valid test fixture");
    assert_eq!(
        tampered.integrity_status(),
        DecisionReplayStatus::PlanMismatch
    );

    let mut tampered = serde_json::to_value(&record).expect("valid test fixture");
    tampered["chosen_action_id"] = serde_json::json!(8);
    let tampered: DecisionRecord = serde_json::from_value(tampered).expect("valid test fixture");
    assert_eq!(
        tampered.integrity_status(),
        DecisionReplayStatus::PlanMismatch
    );

    let mut legacy = record.clone();
    legacy.emulate_legacy_warp_v2();
    let legacy_json = serde_json::to_string(&legacy).expect("valid test fixture");
    assert_eq!(legacy.schema_version, 2);
    assert!(!legacy_json.contains("terminal_evidence_hash"));
    assert_eq!(legacy.integrity_status(), DecisionReplayStatus::Verified);
}
