use crate::adaptive::{
    AdaptivePlayabilityPolicy, BeamConfig, DecisionPrivacy, DecisionRecord, PlannerContext,
    ShadowPrices, WarpDecisionRecordInput, WarpPlanner, WarpPlannerConfig, WarpPlannerInput,
    WarpPlanningDecision,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

pub(super) fn planned() -> (crate::adaptive::PlayabilitySnapshot, WarpPlanningDecision) {
    let state = snapshot(2, 20_000_000, 8_000, 18);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state);
    let config = WarpPlannerConfig {
        beam: BeamConfig::new(2, 8, 64, u64::MAX),
        ..WarpPlannerConfig::default()
    };
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    (state, decision)
}

pub(super) fn record(
    state: &crate::adaptive::PlayabilitySnapshot,
    decision: &WarpPlanningDecision,
) -> DecisionRecord {
    DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 73,
        snapshot: state,
        decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([17; 32]),
    })
}
