use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    ActionKind, Allocation, AllocationPlan, AllocationReason, CandidateUtility, PlannerCapability,
    PlannerCommand, PlannerContext, PreemptionAuthority, RetrievalRequest, WholeBodyContract,
    WholeFetchReason,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn unsupported_media_without_transform_never_claims_playable_gain() {
    let input = snapshot(1, 20_000_000, 8_000, 20);
    let post = input.candidates[0].post.clone();
    let base = AllocationPlan {
        allocations: vec![Allocation {
            post: post.clone(),
            request: RetrievalRequest::FetchWhole {
                contract: WholeBodyContract::Capped {
                    maximum_bytes: 3_750_000,
                },
                reason: WholeFetchReason::PlannedCompletion,
            },
            source: "https://origin.example/media".into(),
            expected_playable_gain_ms: 60_000,
            utility: CandidateUtility {
                view_probability: 1.0,
                additional_playable_ms: 60_000,
                expected_delivery_ms: 1_000,
                score: 60.0,
            },
            authority: PreemptionAuthority::PlaybackCritical,
            commitment_until_ms: 20_000,
            reason: AllocationReason::CurrentStallPrevention,
        }],
        ..AllocationPlan::default()
    };
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_capability(&post, PlannerCapability::reported(false, None, 1));
    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
    let whole = generated
        .actions
        .iter()
        .find(|item| matches!(item.node.kind, ActionKind::FetchWhole { .. }))
        .expect("whole-object action");

    assert_eq!(whole.node.forecast.ready_playback_ms, 0);
    assert!(matches!(
        &whole.command,
        PlannerCommand::Transfer(allocation)
            if allocation.expected_playable_gain_ms == 0
                && allocation.utility.additional_playable_ms == 0
                && allocation.utility.score == 0.0
    ));
    assert!(!generated
        .actions
        .iter()
        .any(|item| matches!(item.node.kind, ActionKind::Transform(_))));
}
