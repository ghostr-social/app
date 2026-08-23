use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate;
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, PlannerContext, WarpActionGenerator,
    BOOTSTRAP_DIRECT_FETCH_BYTES,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn unobserved_unknown_size_complete_file_exposes_probe_and_capped_fetch() {
    let candidate = range_blind_candidate();
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);

    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);

    assert!(generated
        .actions
        .iter()
        .any(|action| action.node.kind == ActionKind::Head));
    assert!(generated.actions.iter().any(|action| {
        action.node.kind
            == ActionKind::FetchWhole {
                maximum_bytes: BOOTSTRAP_DIRECT_FETCH_BYTES,
            }
    }));
}
