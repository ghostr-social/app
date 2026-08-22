use crate::adaptive::{
    ActionKind, DecisionRecord, DecisionReplayStatus, PlannerCommand, PlannerRetryAvailability,
    PlannerRetryEvidence, RecordedPlannerRetryAvailability,
};
use crate::tests::adaptive::decision_record_warp_test_support::{decision, record};
use crate::PostId;

#[test]
fn retry_eligibility_is_recorded_with_exact_time_and_private_identity() {
    let mut decision = decision(
        "secret-post",
        PlannerCommand::Cancel(crate::ActionId::new(4)),
        ActionKind::Cancel(crate::ActionId::new(4)),
    );
    decision.retry_availability = vec![PlannerRetryEvidence::new(
        PostId::new("secret-post"),
        PlannerRetryAvailability::Cooling {
            eligible_at_ms: 42_000,
        },
    )];

    let captured = record(&decision);
    let evidence = &captured.warp_decision.as_ref().unwrap().retry_availability[0];
    assert_ne!(evidence.post_id, "secret-post");
    assert_eq!(
        evidence.availability,
        RecordedPlannerRetryAvailability::Cooling {
            eligible_at_ms: 42_000,
        }
    );
    assert!(!serde_json::to_string(&captured)
        .unwrap()
        .contains("secret-post"));
    assert_eq!(captured.replay(), DecisionReplayStatus::Verified);

    let mut value = serde_json::to_value(captured).unwrap();
    value["warp_decision"]["retry_availability"][0]["availability"]["cooling"]["eligible_at_ms"] =
        serde_json::json!(43_000);
    let tampered: DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
}
