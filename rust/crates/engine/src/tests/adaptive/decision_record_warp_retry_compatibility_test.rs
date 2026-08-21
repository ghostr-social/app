use crate::adaptive::{ActionKind, DecisionRecord, DecisionReplayStatus, PlannerCommand};
use crate::tests::adaptive::decision_record_warp_test_support::{decision, record};
use crate::PostId;

#[test]
fn schema_two_without_retry_evidence_keeps_its_durable_identity() {
    let old = record(&decision(
        "post",
        PlannerCommand::ProbeHead {
            post: PostId::new("post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    ));
    let json = serde_json::to_string(&old).unwrap();
    assert!(!json.contains("retry_availability"));

    let restored: DecisionRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), json);
    assert_eq!(restored.replay(), DecisionReplayStatus::Verified);
}
