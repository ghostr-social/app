use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionRecordInput,
    DecisionReplayStatus, ShadowPrices,
};
use crate::tests::adaptive_support::snapshot;
use sha2::{Digest as _, Sha256};

#[test]
fn schema_one_json_and_replay_identity_remain_stable() {
    let state = snapshot(1, 20_000_000, 8_000, 18);
    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let record = DecisionRecord::capture(DecisionRecordInput {
        sequence: 7,
        snapshot: &state,
        allocation: &plan,
        shadow_prices: ShadowPrices::new(12, 34, 0, 56),
        models: &[],
        privacy: &DecisionPrivacy::from_key([9; 32]),
    });

    let json = serde_json::to_string(&record).expect("valid test fixture");
    let digest = format!("{:x}", Sha256::digest(json.as_bytes()));

    assert_eq!(record.schema_version, 1);
    assert!(!json.contains("warp_decision"));
    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
    assert_eq!(
        digest,
        "d9841ba4de273cd5a7fc1e767b2757d13ba79bdc4967acd2061f526f5a8a6e97"
    );
}

#[test]
fn capability_blocked_state_uses_a_new_replay_schema() {
    let mut state = snapshot(1, 20_000_000, 8_000, 18);
    state.candidates[0].direct_playback_blocked = true;
    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let record = DecisionRecord::capture(DecisionRecordInput {
        sequence: 8,
        snapshot: &state,
        allocation: &plan,
        shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([9; 32]),
    });
    let json = serde_json::to_string(&record).expect("valid test fixture");

    assert_eq!(record.schema_version, 4);
    assert!(json.contains("direct_playback_blocked"));
    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
}
