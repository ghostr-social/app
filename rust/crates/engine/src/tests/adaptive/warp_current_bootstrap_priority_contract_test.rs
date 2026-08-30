use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, AllocationPlan, ControlMode, MediaLayout,
    PlannerCommand, PlannerContext, PlayerPreparation, PreemptionAuthority, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn unresolved_visible_current_commits_a_byte_prefix_before_head() {
    let input = unresolved_current();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    let selected = decision.selected.expect("current bootstrap commitment");
    assert_eq!(selected.node.post, input.playback.current);
    assert!(
        matches!(selected.node.kind, ActionKind::Prefix(range) if !range.is_empty()),
        "selected advisory work instead of bytes: {:?}",
        selected.node.kind
    );
}
#[test]
fn canonical_current_prefix_is_playback_critical_in_every_control_mode() {
    let input = unresolved_current();
    let context = PlannerContext::explicitly_unavailable(&input);
    let authorities: Vec<_> = [
        ControlMode::Normal,
        ControlMode::Safety,
        ControlMode::Emergency,
    ]
    .into_iter()
    .map(|mode| prefix_authority(&input, &context, mode))
    .collect();

    assert_eq!(authorities, [PreemptionAuthority::PlaybackCritical; 3]);
}

#[test]
fn slack_future_with_unresolved_metadata_keeps_head_eligible() {
    let mut input = snapshot(2, 20_000_000, 20_000, 0);
    let future = &mut input.candidates[1];
    future.layout = MediaLayout::Unknown;
    future
        .present
        .push(future.startup.as_ref().expect("startup").ranges()[0]);
    let post = future.post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    assert_eq!(base.mode, ControlMode::Normal);
    let context = PlannerContext::explicitly_unavailable(&input);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    let head = decision
        .generated
        .actions
        .iter()
        .find(|action| action.node.post == post && action.node.kind == ActionKind::Head)
        .expect("future HEAD candidate");
    assert!(decision.admissible_action_ids.contains(&head.node.id));
}

fn unresolved_current() -> crate::adaptive::PlayabilitySnapshot {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates[0].layout = MediaLayout::Unknown;
    input.candidates[0].total_bytes = None;
    input.candidates[0].evidence = Default::default();
    input.candidates[0].player_preparation = PlayerPreparation::Unverified;
    input
}

fn prefix_authority(
    input: &crate::adaptive::PlayabilitySnapshot,
    context: &PlannerContext,
    mode: ControlMode,
) -> PreemptionAuthority {
    let base = AllocationPlan {
        mode,
        ..AllocationPlan::default()
    };
    WarpActionGenerator::generate(input, &base, &OriginModel::default(), context)
        .actions
        .into_iter()
        .find_map(|action| match (action.node.kind, action.command) {
            (ActionKind::Prefix(_), PlannerCommand::Transfer(work)) => Some(work.authority),
            _ => None,
        })
        .expect("current prefix")
}
