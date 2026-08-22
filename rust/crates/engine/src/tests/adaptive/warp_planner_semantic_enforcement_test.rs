use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, SemanticScore, WarpPlanner, WarpPlannerConfig,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn planner_never_executes_an_ordinary_action_outside_semantic_admission() {
    let mut input = snapshot(3, 20_000_000, 20_000, 0);
    let current_start = input.candidates[0].playable_ranges[0].bytes;
    input.candidates[0].present.push(current_start);
    let allowed_start = input.candidates[1].playable_ranges[0].bytes;
    input.candidates[1].present.push(allowed_start);
    input.candidates[0].retrieval_eligible = false;
    input.candidates[1].view_probability = crate::adaptive::ViewProbability::new(0.75).unwrap();
    input.candidates[2].view_probability = crate::adaptive::ViewProbability::new(1.0).unwrap();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    assert_eq!(base.mode, crate::adaptive::ControlMode::Normal);
    let allowed = input.candidates[1].post.clone();
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_semantic(
            input.candidates[0].post.clone(),
            SemanticScore::Known(1_000),
        )
        .with_semantic(allowed.clone(), SemanticScore::Known(1_000))
        .with_semantic(input.candidates[2].post.clone(), SemanticScore::Known(1));
    let config = WarpPlannerConfig {
        semantic_top_k: 3,
        ..WarpPlannerConfig::default()
    };
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert_eq!(
        decision.selected.expect("admissible action").node.post,
        allowed
    );
}
