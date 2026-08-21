use super::apply;
use crate::adaptive::{
    ActionKind, AllocationPlan, ControlMode, PlannerCapability, PlannerCommand, PlannerContext,
    TransformCapability, TransformKind, WarpActionGenerator, WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn transform_required_media_reserves_fetch_then_transform() {
    let mut input = snapshot(1, 20_000_000, 8_000, 20);
    input.commitment_ms = 100_000;
    let base = AllocationPlan {
        mode: ControlMode::Safety,
        ..AllocationPlan::default()
    };
    let post = input.candidates[0].post.clone();
    let mut context = PlannerContext::explicitly_unavailable(&input).with_capability(
        post,
        PlannerCapability::reported(
            false,
            Some(TransformCapability::new(TransformKind::Remux, 10, 128_000)),
            1,
        ),
    );
    context.limits.cpu_ms = 10;
    context.limits.request_tokens = 1;
    context.limits.per_origin_requests = 1;
    let origins = OriginModel::default();
    let planner_input = WarpPlannerInput::new(&input, &base, &origins, &context);
    let generated = WarpActionGenerator::generate(&input, &base, &origins, &context);
    let frontier: Vec<_> = generated
        .actions
        .iter()
        .map(|item| item.node.clone())
        .collect();
    let result = apply(
        &planner_input,
        &frontier,
        &WarpPlannerConfig::default().with_rescue_thresholds(0, 0),
        context.limits.network_burst_bytes,
    );
    let whole = frontier
        .iter()
        .find(|node| matches!(node.kind, ActionKind::FetchWhole { .. }))
        .unwrap();
    let transform = frontier
        .iter()
        .find(|node| matches!(node.kind, ActionKind::Transform(_)))
        .unwrap();
    let whole_command = generated
        .actions
        .iter()
        .find(|item| item.node.id == whole.id)
        .map(|item| &item.command)
        .unwrap();

    assert_eq!(whole.forecast.ready_playback_ms, 0);
    assert!(matches!(
        whole_command,
        PlannerCommand::Transfer(allocation) if allocation.expected_playable_gain_ms == 0
    ));
    assert_eq!(transform.requires, [whole.id]);
    assert!(!result.reserve.degraded);
    assert_eq!(result.reserve.reserved_request_slots, 1);
    assert_eq!(
        result.reserve.reserved_network_bytes,
        whole.resources.network_bytes
    );
}
