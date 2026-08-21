use super::{DecisionRecord, LEGACY_SCHEMA_VERSION, WARP_SCHEMA_VERSION};
use crate::adaptive::decision::plan;
use crate::adaptive::decision::state::ReplayState;
use crate::adaptive::{AdaptivePlayabilityPolicy, DecisionReplayStatus, VerifiedWarpReplay};
use serde::Serialize;
use sha2::{Digest, Sha256};

mod trace;

const WARP_STATE_DOMAIN: &[u8] = b"ghostr-warp-v2-state\0";
const WARP_DECISION_DOMAIN: &[u8] = b"ghostr-warp-v2-decision\0";

pub(super) fn status(record: &DecisionRecord) -> DecisionReplayStatus {
    match (record.schema_version, record.warp_decision.is_some()) {
        (LEGACY_SCHEMA_VERSION, false) => legacy(record),
        (WARP_SCHEMA_VERSION, true) => replay_status(warp(record)),
        _ => DecisionReplayStatus::UnsupportedSchema,
    }
}

pub(super) fn warp(record: &DecisionRecord) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
    if record.schema_version != WARP_SCHEMA_VERSION || record.warp_decision.is_none() {
        return Err(DecisionReplayStatus::UnsupportedSchema);
    }
    if warp_state_identity(&record.replay_state).0 != record.state_hash {
        return Err(DecisionReplayStatus::StateHashMismatch);
    }
    if record.replay_plan_hash != warp_identity(record) {
        return Err(DecisionReplayStatus::PlanMismatch);
    }
    trace::reconstruct(record)
}

pub(super) fn state_identity(state: &ReplayState) -> (String, u64) {
    let encoded = serde_json::to_vec(state).expect("replay state is serializable");
    identity(Sha256::digest(encoded))
}

pub(super) fn warp_state_identity(state: &ReplayState) -> (String, u64) {
    let encoded = serde_json::to_vec(state).expect("replay state is serializable");
    let mut digest = Sha256::new();
    digest.update(WARP_STATE_DOMAIN);
    digest.update(encoded);
    let (hash, seed) = identity(digest.finalize());
    (format!("warp-v2-state:{hash}"), seed)
}

pub(super) fn warp_identity(record: &DecisionRecord) -> String {
    let immutable = (
        record.schema_version,
        record.sequence,
        &record.state_hash,
        &record.admissible_candidates,
        &record.retained_plans,
        &record.pruned,
        &record.model_quantiles,
        record.shadow_prices,
        &record.chosen_action,
        record.random_seed,
        &record.warp_decision,
    );
    tagged_hash(&immutable)
}

fn legacy(record: &DecisionRecord) -> DecisionReplayStatus {
    if state_identity(&record.replay_state).0 != record.state_hash {
        return DecisionReplayStatus::StateHashMismatch;
    }
    let replayed = AdaptivePlayabilityPolicy.plan(&record.replay_state.snapshot());
    match plan::replay_hash(&replayed) == record.replay_plan_hash {
        true => DecisionReplayStatus::Verified,
        false => DecisionReplayStatus::PlanMismatch,
    }
}

fn replay_status(result: Result<VerifiedWarpReplay, DecisionReplayStatus>) -> DecisionReplayStatus {
    result.map_or_else(|status| status, |_| DecisionReplayStatus::Verified)
}

fn tagged_hash(data: &impl Serialize) -> String {
    let encoded = serde_json::to_vec(data).expect("decision identity is serializable");
    let mut digest = Sha256::new();
    digest.update(WARP_DECISION_DOMAIN);
    digest.update(encoded);
    format!("warp-v2-decision:{}", hex(&digest.finalize()))
}

fn identity(digest: impl AsRef<[u8]>) -> (String, u64) {
    let digest = digest.as_ref();
    let mut seed = [0; 8];
    seed.copy_from_slice(&digest[..8]);
    (hex(digest), u64::from_be_bytes(seed).max(1))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
