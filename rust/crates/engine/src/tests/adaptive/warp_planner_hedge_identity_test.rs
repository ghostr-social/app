use crate::adaptive::{
    ActionKind, ActivePlannerContext, AllocationPlan, HedgeInput, IdentityProof, InFlightAction,
    PlannerCommand, PlannerContext, PromotionGrant, RetrievalRequest, WarpActionGenerator,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

const PRIMARY: &str = "https://primary.example/media";
const ALTERNATE: &str = "https://alternate.example/media";

#[test]
fn hedge_requires_the_exact_primary_and_effective_request() {
    let primary = ActionId::new(17);
    let range = ByteRange::new(0, 64_000);
    let forged_primary = hedge(ActionId::new(18), ActionKind::FetchRange(range));
    let forged_request = hedge(primary, ActionKind::FetchRange(ByteRange::new(1, 2)));

    assert!(!generates_hedge(primary, range, forged_primary));
    assert!(!generates_hedge(primary, range, forged_request));
}

#[test]
fn promoted_range_cannot_hide_a_large_network_envelope() {
    let primary = ActionId::new(17);
    let range = ByteRange::new(0, 64_000);
    let request = RetrievalRequest::FetchRange {
        bytes: range,
        promotion: Some(PromotionGrant {
            maximum_bytes: 2 * 1024 * 1024,
            valid_until_ms: 20_000,
        }),
    };
    let input = hedge(primary, ActionKind::FetchRange(range));

    assert!(!generates_for_request(primary, request, input));
}

fn generates_hedge(primary: ActionId, range: ByteRange, input: HedgeInput) -> bool {
    generates_for_request(
        primary,
        RetrievalRequest::FetchRange {
            bytes: range,
            promotion: None,
        },
        input,
    )
}

fn generates_for_request(primary: ActionId, request: RetrievalRequest, input: HedgeInput) -> bool {
    let mut snapshot = snapshot(1, 8_000_000, 1_000, 20);
    let bytes = request.requested_bytes();
    let mut active = InFlightAction::range(primary, bytes, PRIMARY, 20_000, true);
    active.request = request;
    snapshot.candidates[0].in_flight.push(active);
    let post = snapshot.candidates[0].post.clone();
    let active = ActivePlannerContext::new(primary, post).with_hedge(
        input,
        IdentityProof::VerifiedHash([3; 32]),
        ALTERNATE,
    );
    let context = PlannerContext::explicitly_unavailable(&snapshot).with_active(active);
    WarpActionGenerator::generate(
        &snapshot,
        &AllocationPlan::default(),
        &OriginModel::default(),
        &context,
    )
    .actions
    .iter()
    .any(|action| matches!(action.command, PlannerCommand::Hedge { .. }))
}

fn hedge(primary: ActionId, action: ActionKind) -> HedgeInput {
    HedgeInput::new(primary, action)
        .with_timing(1_000, 900)
        .with_value(5_000, 1_000)
}
