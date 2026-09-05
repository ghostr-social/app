use super::warp_request_envelope_fixture::{request, RESERVED_BYTES, SOURCE};
use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    ActionKind, ActivePlannerContext, HedgeInput, IdentityProof, InFlightAction,
};
use crate::adaptive::{AllocationPlan, PlannerCommand, PlannerContext};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};
const MIRROR: &str = "https://mirror.example/media";

#[test]
fn a_range_grant_and_past_hash_do_not_authorize_a_duplicate_range() {
    let mut input = snapshot(1, 8_000_000, 1_000, 20);
    let active_id = ActionId::new(17);
    let range = ByteRange::new(0, 64_000);
    let mut active = InFlightAction::range(active_id, range, SOURCE, 20_000, true);
    active.request = request();
    input.candidates[0].in_flight.push(active);
    let hedge = HedgeInput::new(active_id, ActionKind::FetchRange(range))
        .with_timing(1_000, 900)
        .with_value(5_000, 1_000)
        .with_network_envelope(RESERVED_BYTES);
    let post = input.candidates[0].post.clone();
    let active = ActivePlannerContext::new(active_id, post).with_hedge(
        hedge,
        IdentityProof::VerifiedHash([3; 32]),
        MIRROR,
    );
    let context = PlannerContext::explicitly_unavailable(&input).with_active(active);
    let generated = WarpActionGenerator::generate(
        &input,
        &AllocationPlan::default(),
        &OriginModel::default(),
        &context,
    );
    let action = generated
        .actions
        .iter()
        .find(|action| matches!(&action.command, PlannerCommand::Hedge { transfer, .. } if transfer.request == request()));
    assert!(
        action.is_none(),
        "new ranges require independent block authentication"
    );
}
