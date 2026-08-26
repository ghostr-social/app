use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    ActionKind, ActivePlannerContext, AdaptivePlayabilityPolicy, HedgeInput, IdentityProof,
    InFlightAction, PlannerCommand, PlannerContext, PromotionGrant, RetrievalRequest,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::{healthy_origin, snapshot};
use crate::{ActionId, ByteRange, PostId};

#[test]
fn cancelling_action_emits_no_control_promotion_or_hedge() {
    let mut input = snapshot(1, 8_000_000, 20_000, 20);
    let action = ActionId::new(42);
    let post = input.candidates[0].post.clone();
    let mut in_flight = InFlightAction::range(
        action,
        ByteRange::new(0, 64_000),
        "https://origin.example/media",
        20_000,
        true,
    );
    in_flight.cancelling = true;
    in_flight.request = RetrievalRequest::FetchRange {
        bytes: ByteRange::new(0, 64_000),
        promotion: Some(PromotionGrant {
            maximum_bytes: 800_000,
            valid_until_ms: 20_000,
        }),
    };
    input.candidates[0].origins.push(healthy_origin(
        "https://mirror.example/media",
        7_000_000,
        60,
    ));
    input.candidates[0].in_flight.push(in_flight);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let hedge = HedgeInput::new(action, ActionKind::FetchRange(ByteRange::new(0, 64_000)))
        .with_timing(1_000, 900)
        .with_value(5_000, 1_000);
    let active = ActivePlannerContext::new(action, post)
        .mark_cancelling()
        .with_continuation_advantage(-100_000)
        .with_hedge(
            hedge,
            IdentityProof::VerifiedHash([3; 32]),
            "https://mirror.example/media",
        );
    let detached = ActivePlannerContext::new(ActionId::new(43), PostId::new("detached"))
        .mark_cancelling()
        .with_continuation_advantage(-100_000);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_active(active)
        .with_active(detached);

    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);

    assert!(generated.active_controls.is_empty());
    assert!(!generated.actions.iter().any(|item| matches!(
        item.command,
        PlannerCommand::Cancel(_) | PlannerCommand::Promote { .. } | PlannerCommand::Hedge { .. }
    )));
}
