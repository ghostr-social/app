use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, AdaptivePlayabilityPolicy, GeneratedAction,
    PlannerCommand, PlannerContext, PlannerLimits, ResourceCost, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, PostId};

pub(super) const OBSERVED_AT_MS: u64 = 10_000;

pub(super) fn planner(capacity: u64, refill_per_second: u64) -> WarpPlanner {
    let input = snapshot(1, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input).with_limits(PlannerLimits {
        network_burst_bytes: capacity,
        network_rate_bytes_per_second: refill_per_second,
        cpu_ms: 10,
        request_tokens: 2,
        per_origin_requests: 2,
    });
    let mut planner = WarpPlanner::default();
    planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    planner
}

pub(super) fn selected(envelope: ResourceCost) -> GeneratedAction {
    let active = ActionId::new(7);
    GeneratedAction {
        node: ActionNode::new(
            1,
            PostId::new("selected"),
            ActionKind::Cancel(active),
            ActionValue::from_net_micros(0),
        )
        .with_resources(envelope),
        command: PlannerCommand::Cancel(active),
    }
}
