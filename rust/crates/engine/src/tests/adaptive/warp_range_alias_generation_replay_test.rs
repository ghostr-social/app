use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionReplayStatus,
    PlannerCapability, PlannerCommand, PlannerContext, RetrievalRequest, ShadowPrices,
    WarpDecisionRecordInput, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::ByteRange;

#[path = "warp_range_alias_generation_replay_fixture.rs"]
mod fixture;
use fixture::partial_state;

const TOTAL: u64 = 293_999;
const RANGE_START: u64 = 65_536;
const RANGE_END: u64 = 131_072;

#[test]
fn marker_free_generation_keeps_historical_independent_range_aliases() {
    let state = partial_state();
    let decision = plan(&state, true);
    let matching_ranges = decision.generated.actions.iter().filter(|action| {
        matches!(&action.command, PlannerCommand::Transfer(allocation)
            if matches!(allocation.request,
                RetrievalRequest::FetchRange { bytes, .. }
                    if bytes == ByteRange::new(RANGE_START, RANGE_END)))
    });
    assert_eq!(
        matching_ranges.count(),
        3,
        "{:#?}",
        decision.generated.actions
    );
    assert!(decision.generated.actions.iter().any(|action| {
        matches!(&action.command, PlannerCommand::Transfer(allocation)
            if matches!(allocation.request,
                RetrievalRequest::FetchRange { promotion: Some(_), .. }))
    }));
    assert!(decision
        .generated
        .actions
        .iter()
        .any(|action| { matches!(action.node.kind, ActionKind::FetchWhole { .. }) }));
    let encoded = encoded_record(&state, &decision);
    assert!(!encoded.contains("range_alias_generation_policy"));
    let restored: DecisionRecord = serde_json::from_str(&encoded).expect("legacy record");
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}

#[test]
fn current_generation_records_promotable_dominance() {
    let state = partial_state();
    let decision = plan(&state, false);
    assert_eq!(decision.generated.actions.len(), 1);
    let encoded = encoded_record(&state, &decision);
    assert!(encoded.contains("\"range_alias_generation_policy\":\"promotable_dominance\""));
    let restored: DecisionRecord = serde_json::from_str(&encoded).expect("current record");
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}

fn plan(
    state: &crate::adaptive::PlayabilitySnapshot,
    legacy: bool,
) -> crate::adaptive::WarpPlanningDecision {
    let base = AdaptivePlayabilityPolicy.plan(state);
    let context = PlannerContext::explicitly_unavailable(state).with_capability(
        &state.candidates[1].post,
        PlannerCapability::reported(true, None, 1),
    );
    let origins = OriginModel::default();
    let input = WarpPlannerInput::new(state, &base, &origins, &context);
    if legacy {
        WarpPlanner::default().plan_legacy_range_aliases_for_test(input)
    } else {
        WarpPlanner::default().plan(input)
    }
}

fn encoded_record(
    state: &crate::adaptive::PlayabilitySnapshot,
    decision: &crate::adaptive::WarpPlanningDecision,
) -> String {
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 1,
        snapshot: state,
        decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([9; 32]),
    });
    serde_json::to_string(&record).expect("decision record")
}
