use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, HeadProbeHistory, MediaLayout, PlannerCommand,
    PlannerContext, WarpActionGenerator,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn head_probe_generation_follows_representation_history() {
    let mut input = snapshot(1, 8_000_000, 0, 0);
    input.candidates[0].layout = MediaLayout::Unknown;
    let post = input.candidates[0].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    for (history, expected_head) in [
        (HeadProbeHistory::Unobserved, true),
        (HeadProbeHistory::Active, false),
        (HeadProbeHistory::Completed, false),
    ] {
        let context = PlannerContext::explicitly_unavailable(&input)
            .with_head_probe_history(post.clone(), history);
        let generated =
            WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);
        assert_eq!(
            generated
                .actions
                .iter()
                .any(|item| item.node.kind == ActionKind::Head),
            expected_head,
            "history: {history:?}"
        );
        assert!(generated
            .actions
            .iter()
            .any(|item| matches!(item.command, PlannerCommand::Transfer(_))));
    }
}
