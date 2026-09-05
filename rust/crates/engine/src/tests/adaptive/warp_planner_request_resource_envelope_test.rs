use super::warp_request_envelope_fixture::{allocation, assert_immediate, request};
use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{AllocationPlan, PlannerCommand, PlannerContext};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn transfer_charges_only_the_immediate_range_before_promotion() {
    let input = snapshot(1, 8_000_000, 1_000, 20);
    let allocation = allocation(input.candidates[0].post.clone());
    let base = AllocationPlan {
        allocations: vec![allocation],
        ..AllocationPlan::default()
    };
    let context = PlannerContext::explicitly_unavailable(&input);
    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
    let action = generated.actions.iter().find(|action| {
        matches!(&action.command, PlannerCommand::Transfer(work) if work.request == request())
    });
    assert_immediate(action.expect("promotable transfer"));
}
