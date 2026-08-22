mod cumulative_chance_test;
mod dependency_path_test;
mod deterministic_choice_test;
mod infeasible_path_test;
mod invalid_dependency_test;
mod joint_chance_test;
mod semantic_dependency_test;
mod shared_dependency_test;

use super::{select, RescueInputs, RescuePlan};
use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, AllocationPlan, CompletionTimes,
    ControlMode, HardBudget, PlannerContext, ResourceCost, SemanticAdmission, SemanticDecision,
    WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

fn select_path(frontier: &[ActionNode], limits: ResourceCost) -> Option<RescuePlan> {
    select_path_at(frontier, limits, 9_500)
}

fn select_path_at(
    frontier: &[ActionNode],
    limits: ResourceCost,
    threshold_bps: u16,
) -> Option<RescuePlan> {
    let input = snapshot(1, 10_000_000, 1_000, 0);
    let base = AllocationPlan {
        mode: ControlMode::Safety,
        ..AllocationPlan::default()
    };
    let origins = OriginModel::default();
    let context = PlannerContext::explicitly_unavailable(&input);
    let planner_input = WarpPlannerInput::new(&input, &base, &origins, &context);
    let semantic = vec![SemanticDecision {
        post: PostId::new("p0"),
        admission: SemanticAdmission {
            admissible: true,
            rescue: false,
            rank_displacement: 0,
            censor: None,
        },
    }];
    let budget = HardBudget::new(limits, 2);
    select(RescueInputs {
        input: &planner_input,
        frontier,
        semantic: &semantic,
        config: &WarpPlannerConfig::default().with_rescue_thresholds(threshold_bps, threshold_bps),
        budget: &budget,
    })
}

fn node(id: u16, resources: ResourceCost, ready_ms: u64, requires: &[u16]) -> ActionNode {
    let kind = ActionKind::Transform(crate::adaptive::TransformKind::Remux);
    let mut node = ActionNode::new(id, PostId::new("p0"), kind, ActionValue::default())
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

fn exact_limits() -> ResourceCost {
    ResourceCost::new(60, 90, 5, 1)
}
