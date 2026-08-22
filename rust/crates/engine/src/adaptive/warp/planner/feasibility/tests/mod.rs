mod actionable_semantic_test;
mod degraded_reserve_test;
mod hard_budget_storage_margin_test;
mod semantic_readiness_test;
mod transform_rescue_test;
mod unsupported_playback_test;
mod zero_request_reserve_test;

use super::{apply, FeasibleActions};
use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, AllocationPlan, CompletionTimes,
    ControlMode, PlannerContext, PlannerLimits, ResourceCost, StorageSnapshot, TransformKind,
    WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

fn feasible(frontier: &[ActionNode], limits: ResourceCost) -> FeasibleActions {
    let mut input = snapshot(1, 10_000_000, 1_000, 0);
    input.storage = StorageSnapshot::new(limits.storage_bytes, 0);
    let base = AllocationPlan {
        mode: ControlMode::Safety,
        ..AllocationPlan::default()
    };
    let origins = OriginModel::default();
    let context = PlannerContext::explicitly_unavailable(&input).with_limits(PlannerLimits {
        network_burst_bytes: limits.network_bytes,
        network_rate_bytes_per_second: limits.network_bytes.max(1),
        cpu_ms: limits.cpu_ms,
        request_tokens: limits.requests,
        per_origin_requests: 2,
    });
    let planner_input = WarpPlannerInput::new(&input, &base, &origins, &context);
    let config = WarpPlannerConfig::default().with_rescue_thresholds(9_000, 9_000);
    apply(&planner_input, frontier, &config, limits.network_bytes)
}

fn node(id: u16, resources: ResourceCost, ready_ms: u64, requires: &[u16]) -> ActionNode {
    let mut node = ActionNode::new(
        id,
        PostId::new("p0"),
        ActionKind::Transform(TransformKind::Remux),
        ActionValue::default(),
    )
    .with_resources(resources)
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(10, 10, 10, 10),
        10_000,
        ready_ms,
    ))
    .requiring(requires);
    if resources.requests > 0 {
        node = node.with_origin("https://origin.example/media");
    }
    node
}
