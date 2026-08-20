use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, HeadProbeHistory, MediaLayout, PlannerCommand,
    PlannerContext, WarpActionGenerator,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn completed_head_probe_is_retired_while_body_work_remains() {
    let mut input = snapshot(1, 8_000_000, 0, 0);
    input.candidates[0].layout = MediaLayout::Unknown;
    let post = input.candidates[0].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(post, HeadProbeHistory::Completed);

    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);

    assert!(generated
        .actions
        .iter()
        .all(|item| item.node.kind != ActionKind::Head));
    assert!(generated
        .actions
        .iter()
        .any(|item| matches!(item.command, PlannerCommand::Transfer(_))));
}
