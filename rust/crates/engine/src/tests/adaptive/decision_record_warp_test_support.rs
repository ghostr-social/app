use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, Allocation, AllocationReason,
    CandidateUtility, DecisionPrivacy, DecisionRecord, GeneratedAction, GeneratedActions,
    PlannerCommand, PlayabilitySnapshot, ReserveConstraint, ResourceCost, ResourcePrices,
    RetrievalRequest, SearchDecision, ShadowPrices, TwinEvaluation, WarpDecisionRecordInput,
    WarpPlanningDecision,
};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

pub(super) fn decision(
    post: &str,
    command: PlannerCommand,
    kind: ActionKind,
) -> WarpPlanningDecision {
    let node = ActionNode::new(
        7,
        PostId::new(post),
        kind,
        ActionValue::from_net_micros(8_000),
    )
    .with_resources(ResourceCost::new(64, 32, 4, 1))
    .with_forecast(ActionForecast::default());
    let selected = GeneratedAction { node, command };
    WarpPlanningDecision {
        generated: GeneratedActions {
            actions: vec![selected.clone()],
            ladders: Vec::new(),
            active_controls: Vec::new(),
        },
        selected: Some(selected),
        additional_request_slot_demanded: false,
        search: SearchDecision::default(),
        evaluation: Some(TwinEvaluation {
            expected_score_micros: 7_000,
            expected_visible_delay_ms: 10,
            p95_visible_delay_ms: 20,
            p99_visible_delay_ms: 30,
            cvar_visible_delay_ms: 40,
            on_time_probability_bps: 9_500,
            expected_ready_coverage_ms: 5_000,
            expected_cache_bytes: 32,
            common_random_seed: 99,
        }),
        admissible_action_ids: vec![7],
        pruned_action_ids: Vec::new(),
        reserve: ReserveConstraint::default(),
        semantic: Vec::new(),
        prices: ResourcePrices {
            network_micros: 1,
            storage_micros: 2,
            cpu_micros: 3,
            request_micros: 4,
        },
    }
}

pub(super) fn record(decision: &WarpPlanningDecision) -> DecisionRecord {
    let state = snapshot(1, 20_000_000, 8_000, 18);
    record_for(decision, &state)
}

pub(super) fn record_for(
    decision: &WarpPlanningDecision,
    state: &PlayabilitySnapshot,
) -> DecisionRecord {
    DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 9,
        snapshot: state,
        decision,
        legacy_shadow_prices: ShadowPrices::new(10, 20, 30, 40),
        models: &[],
        privacy: &DecisionPrivacy::from_key([5; 32]),
    })
}

pub(super) fn allocation(source: &str, request: RetrievalRequest) -> Allocation {
    Allocation {
        post: PostId::new("secret-post"),
        request,
        source: source.into(),
        expected_playable_gain_ms: 1_000,
        utility: CandidateUtility {
            view_probability: 1.0,
            additional_playable_ms: 1_000,
            expected_delivery_ms: 10,
            score: 1.0,
        },
        authority: crate::adaptive::PreemptionAuthority::PlaybackCritical,
        commitment_until_ms: 1_000,
        reason: AllocationReason::MediaBootstrap,
    }
}
