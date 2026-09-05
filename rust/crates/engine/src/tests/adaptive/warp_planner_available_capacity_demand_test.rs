use crate::adaptive::{
    AdaptivePlayabilityPolicy, AllocationPlan, BeamConfig, ControlMode, PlannerContext,
    PlayabilitySnapshot, ViewProbability, WarpPlanner, WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::{ColdStartPrior, ColdStartSelector, OriginModel, RequestMethod};
use crate::tests::adaptive_support::snapshot;

#[test]
fn selected_action_reports_demand_for_an_independent_second_slot() {
    let scenario = scenario();
    let lookahead = decision(&scenario, 2);
    assert_eq!(independent_requests(&lookahead), 2);
    let selected = decision(&scenario, 1);

    let selected_action = selected.selected.as_ref().expect("committed action");
    assert_eq!(selected_action.node.resources.requests, 1);
    assert_eq!(selected.search.committed_actions, 1);
    assert!(selected.additional_request_slot_demanded);
}

fn independent_requests(decision: &crate::adaptive::WarpPlanningDecision) -> usize {
    let chosen = &decision
        .search
        .chosen_plan
        .as_ref()
        .expect("fixture")
        .action_ids;
    decision
        .generated
        .actions
        .iter()
        .filter(|action| chosen.contains(&action.node.id))
        .filter(|action| action.node.requires.is_empty() && action.node.resources.requests > 0)
        .count()
}

struct Scenario {
    input: PlayabilitySnapshot,
    base: AllocationPlan,
    origins: OriginModel,
}

fn scenario() -> Scenario {
    let mut input = snapshot(3, 100_000_000, 20_000, 0);
    input.network.connection_capacity = 1;
    input.network.connection_ceiling = 2;
    input.network.per_authority_request_limit = 2;
    input.candidates[0].retrieval_eligible = false;
    for candidate in &mut input.candidates {
        candidate.present.push(candidate.playable_ranges[0].bytes);
        candidate.view_probability = ViewProbability::new(1.0).expect("fixture");
    }
    let base = AdaptivePlayabilityPolicy.plan(&input);
    assert_eq!(base.mode, ControlMode::Normal);
    Scenario {
        input,
        base,
        origins: reliable_origin(),
    }
}

fn decision(scenario: &Scenario, request_tokens: u16) -> crate::adaptive::WarpPlanningDecision {
    let mut context = PlannerContext::explicitly_unavailable(&scenario.input);
    context.limits.request_tokens = request_tokens;
    planner().plan(WarpPlannerInput::new(
        &scenario.input,
        &scenario.base,
        &scenario.origins,
        &context,
    ))
}

fn planner() -> WarpPlanner {
    WarpPlanner::new(WarpPlannerConfig {
        beam: BeamConfig::new(4, 32, 256, u64::MAX),
        ..WarpPlannerConfig::default().with_lookahead()
    })
}

fn reliable_origin() -> OriginModel {
    let mut model = OriginModel::default();
    let mut prior = ColdStartPrior::new(100.0, 0.1, 1, 100_000_000);
    prior.range_alpha = 100.0;
    prior.range_beta = 0.1;
    model.register_cold_start(
        ColdStartSelector::default().with_method(RequestMethod::RangeGet),
        prior,
    );
    model
}
