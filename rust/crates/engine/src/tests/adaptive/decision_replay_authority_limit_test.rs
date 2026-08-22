use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionRecordInput,
    DecisionReplayStatus, ShadowPrices,
};
use crate::tests::adaptive_support::snapshot;
use sha2::{Digest, Sha256};

#[test]
fn replay_without_the_authority_limit_preserves_its_legacy_state_hash() {
    let mut state = snapshot(2, 20_000_000, 8_000, 0);
    state.network.connection_capacity = 2;
    state.network.connection_ceiling = 2;
    state.network.per_authority_request_limit = 1;
    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let record = DecisionRecord::capture(DecisionRecordInput {
        sequence: 1,
        snapshot: &state,
        allocation: &plan,
        shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([4; 32]),
    });
    let field = ",\"per_authority_request_limit\":1";
    let mut json = serde_json::to_string(&record).unwrap();
    assert!(json.contains(field));
    json = json.replace(field, "");
    let replay = replay_json(&json);
    let digest = Sha256::digest(replay.as_bytes());
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    json = json.replace(&record.state_hash, &hash);

    let restored: DecisionRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.replay(), DecisionReplayStatus::Verified);
    assert!(!serde_json::to_string(&restored)
        .unwrap()
        .contains("per_authority_request_limit"));
}

fn replay_json(record: &str) -> &str {
    record
        .split_once("\"replay_state\":")
        .and_then(|(_, tail)| tail.split_once(",\"replay_plan_hash\""))
        .map(|(state, _)| state)
        .expect("serialized replay state")
}
