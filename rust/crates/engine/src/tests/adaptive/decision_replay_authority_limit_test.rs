use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionRecordInput,
    DecisionReplayStatus, ShadowPrices,
};
use crate::tests::adaptive_support::snapshot;
use sha2::{Digest as _, Sha256};

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
    let mut json = serde_json::to_string(&record).expect("valid test fixture");
    assert!(json.contains(field));
    json = json.replace(field, "");
    let replay = replay_json(&json);
    let digest = Sha256::digest(replay.as_bytes());
    let hash = hex(&digest);
    json = json.replace(&record.state_hash, &hash);

    let restored: DecisionRecord = serde_json::from_str(&json).expect("valid test fixture");

    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(!serde_json::to_string(&restored)
        .expect("valid test fixture")
        .contains("per_authority_request_limit"));
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}

fn replay_json(record: &str) -> &str {
    record
        .split_once("\"replay_state\":")
        .and_then(|(_, tail)| tail.split_once(",\"replay_plan_hash\""))
        .map(|(state, _)| state)
        .expect("serialized replay state")
}
