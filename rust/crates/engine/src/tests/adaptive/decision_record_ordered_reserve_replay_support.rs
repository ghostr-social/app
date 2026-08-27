use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, DecisionPrivacy, DecisionRecord, DecisionRecordInput,
    PlayerPreparation, ShadowPrices,
};
use crate::tests::adaptive_support::snapshot;

pub(super) fn gap_state() -> crate::adaptive::PlayabilitySnapshot {
    let mut state = snapshot(6, 3_000_000, 20_000, 60);
    for candidate in &mut state.candidates[1..=4] {
        let startup = candidate.startup.as_ref().expect("startup").ranges()[0];
        candidate.present = vec![startup];
    }
    state.candidates[3].player_preparation = PlayerPreparation::Unverified;
    state
}

pub(super) fn no_gap_state() -> crate::adaptive::PlayabilitySnapshot {
    snapshot(2, 20_000_000, 8_000, 18)
}

pub(super) fn historical_plan(
    state: &crate::adaptive::PlayabilitySnapshot,
) -> crate::adaptive::AllocationPlan {
    let mut plan = AdaptivePlayabilityPolicy.plan(state);
    assert_eq!(plan.mode, ControlMode::Safety);
    plan.mode = ControlMode::Normal;
    plan
}

pub(super) fn legacy_record(
    state: &crate::adaptive::PlayabilitySnapshot,
    plan: &crate::adaptive::AllocationPlan,
    schema: u16,
) -> DecisionRecord {
    let privacy = DecisionPrivacy::from_key([7; 32]);
    let mut record = capture_with_privacy(state, plan, &privacy);
    record.emulate_legacy_policy_record(plan, &privacy, schema);
    record
}

pub(super) fn capture(
    state: &crate::adaptive::PlayabilitySnapshot,
    plan: &crate::adaptive::AllocationPlan,
) -> DecisionRecord {
    capture_with_privacy(state, plan, &DecisionPrivacy::from_key([7; 32]))
}

fn capture_with_privacy(
    state: &crate::adaptive::PlayabilitySnapshot,
    plan: &crate::adaptive::AllocationPlan,
    privacy: &DecisionPrivacy,
) -> DecisionRecord {
    DecisionRecord::capture(DecisionRecordInput {
        sequence: 11,
        snapshot: state,
        allocation: plan,
        shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy,
    })
}

pub(super) fn with_schema(record: &DecisionRecord, schema: u16) -> DecisionRecord {
    let json = serde_json::to_string(record).expect("record");
    let current = format!("\"schema_version\":{}", record.schema_version);
    let requested = format!("\"schema_version\":{schema}");
    serde_json::from_str(&json.replacen(&current, &requested, 1)).expect("legacy record")
}
