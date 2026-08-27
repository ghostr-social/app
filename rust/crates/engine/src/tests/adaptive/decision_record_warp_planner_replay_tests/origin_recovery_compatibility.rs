use super::support::{planned, record};
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn inactive_recovery_fields_keep_the_legacy_replay_identity() {
    let (state, decision) = planned();
    let captured = record(&state, &decision);
    let json = serde_json::to_string(&captured).expect("valid test fixture");
    let restored: DecisionRecord = serde_json::from_str(&json).expect("valid legacy shape");

    assert!(!json.contains("probe_generation"));
    assert!(!json.contains("trial_pending"));
    assert_eq!(
        serde_json::to_string(&restored).expect("stable legacy shape"),
        json
    );
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}
