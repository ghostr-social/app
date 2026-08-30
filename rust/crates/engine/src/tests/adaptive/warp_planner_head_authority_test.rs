use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{AdaptivePlayabilityPolicy, MediaLayout, PlannerCommand, PlannerContext};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

mod warp_current_bootstrap_priority_contract_test {
    include!("warp_current_bootstrap_priority_contract_test.rs");
}
mod warp_current_head_suppression_test {
    include!("warp_current_head_suppression_test.rs");
}

#[test]
fn a_head_probe_uses_the_same_preemption_authority_as_its_candidate_transfer() {
    let mut input = snapshot(2, 8_000_000, 1_000, 20);
    input.candidates[1].layout = MediaLayout::Unknown;
    let future = input.candidates[1].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
    let head = generated
        .actions
        .iter()
        .find_map(|action| match &action.command {
            PlannerCommand::ProbeHead {
                post, authority, ..
            } if post == &future => Some(*authority),
            _ => None,
        });
    let transfer = generated
        .actions
        .iter()
        .find_map(|action| match &action.command {
            PlannerCommand::Transfer(value) if value.post == future => Some(value.authority),
            _ => None,
        });

    assert_eq!(
        head.expect("generated HEAD"),
        transfer.expect("generated transfer")
    );
}
