use super::{
    DecisionRecord, CAPABILITY_SCHEMA_VERSION, LEGACY_SCHEMA_VERSION,
    ORDERED_RESERVE_SCHEMA_VERSION, UNSEALED_WARP_SCHEMA_VERSION, WARP_SCHEMA_VERSION,
};
use crate::adaptive::decision::plan_identity;
use crate::adaptive::decision::replay::VerifiedWarpReplay;
use crate::adaptive::decision::state::ReplayState;
use crate::adaptive::{AdaptivePlayabilityPolicy, DecisionReplayStatus};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

mod trace;

const WARP_STATE_DOMAIN: &[u8] = b"ghostr-warp-v2-state\0";
const WARP_DECISION_DOMAIN: &[u8] = b"ghostr-warp-v2-decision\0";
const TERMINAL_EVIDENCE_DOMAIN: &[u8] = b"ghostr-warp-terminal-evidence\0";

pub(super) fn status(record: &DecisionRecord) -> DecisionReplayStatus {
    match (record.schema_version, record.warp_decision.is_some()) {
        (LEGACY_SCHEMA_VERSION | CAPABILITY_SCHEMA_VERSION, false) => legacy(record),
        (ORDERED_RESERVE_SCHEMA_VERSION, false) => ordered(record),
        (UNSEALED_WARP_SCHEMA_VERSION | WARP_SCHEMA_VERSION | CAPABILITY_SCHEMA_VERSION, true) => {
            replay_status(warp(record))
        }
        _ => DecisionReplayStatus::UnsupportedSchema,
    }
}

pub(super) fn search_status(record: &DecisionRecord) -> DecisionReplayStatus {
    replay_status(warp_search(record))
}

pub(super) fn warp(record: &DecisionRecord) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
    if !is_warp_schema(record.schema_version) || record.warp_decision.is_none() {
        return Err(DecisionReplayStatus::UnsupportedSchema);
    }
    verify_terminal_evidence(record)?;
    if warp_state_identity(&record.replay_state).0 != record.state_hash {
        return Err(DecisionReplayStatus::StateHashMismatch);
    }
    if record.replay_plan_hash != warp_identity(record) {
        return Err(DecisionReplayStatus::PlanMismatch);
    }
    trace::reconstruct(record)
}

pub(super) fn warp_search(
    record: &DecisionRecord,
) -> Result<VerifiedWarpReplay, DecisionReplayStatus> {
    let verified = warp(record)?;
    trace::verify_fresh_search(record)?;
    Ok(verified)
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

pub(super) fn terminal_identity(record: &DecisionRecord) -> String {
    let evidence = (
        record.schema_version,
        record.sequence,
        &record.replay_plan_hash,
        record.chosen_action_id,
        &record.executed_request,
        &record.eventual_outcome,
        &record.actual_resources,
    );
    let encoded = serde_json::to_vec(&evidence).expect("terminal evidence is serializable");
    let mut digest = Sha256::new();
    digest.update(TERMINAL_EVIDENCE_DOMAIN);
    digest.update(encoded);
    format!("warp-terminal-evidence:{}", hex(&digest.finalize()))
}

fn legacy(record: &DecisionRecord) -> DecisionReplayStatus {
    let replayed = AdaptivePlayabilityPolicy.plan_legacy_replay(&record.replay_state.snapshot());
    policy_status(record, plan_identity::legacy(&replayed))
}

fn ordered(record: &DecisionRecord) -> DecisionReplayStatus {
    let replayed = AdaptivePlayabilityPolicy.plan(&record.replay_state.snapshot());
    policy_status(record, plan_identity::ordered(&replayed))
}

fn policy_status(record: &DecisionRecord, replayed_hash: String) -> DecisionReplayStatus {
    if verify_terminal_evidence(record).is_err() {
        return DecisionReplayStatus::PlanMismatch;
    }
    if state_identity(&record.replay_state).0 != record.state_hash {
        return DecisionReplayStatus::StateHashMismatch;
    }
    if replayed_hash == record.replay_plan_hash {
        DecisionReplayStatus::Verified
    } else {
        DecisionReplayStatus::PlanMismatch
    }
}

fn verify_terminal_evidence(record: &DecisionRecord) -> Result<(), DecisionReplayStatus> {
    let sealed = record.schema_version == WARP_SCHEMA_VERSION
        || (record.schema_version == CAPABILITY_SCHEMA_VERSION && record.warp_decision.is_some());
    if sealed && record.terminal_evidence_hash.is_none() {
        return Err(DecisionReplayStatus::PlanMismatch);
    }
    match &record.terminal_evidence_hash {
        Some(expected) if expected != &terminal_identity(record) => {
            Err(DecisionReplayStatus::PlanMismatch)
        }
        _ => Ok(()),
    }
}

fn is_warp_schema(version: u16) -> bool {
    matches!(
        version,
        UNSEALED_WARP_SCHEMA_VERSION | WARP_SCHEMA_VERSION | CAPABILITY_SCHEMA_VERSION
    )
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
    super::super::privacy::hex(bytes)
}
