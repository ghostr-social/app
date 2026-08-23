use super::super::semantic_decisions;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, SemanticScore, WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn startup_ready_incomplete_current_cannot_starve_parallel_ahead_preparation() {
    let mut input = snapshot(2, 20_000_000, 20_000, 0);
    let current = input.candidates[0].post.clone();
    let startup = input.candidates[0].playable_ranges[0].bytes;
    input.candidates[0].present.push(startup);
    let ahead = input.candidates[1].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_semantic(current, SemanticScore::Known(1_000_000))
        .with_semantic(ahead.clone(), SemanticScore::Known(667_000));
    let origins = OriginModel::default();
    let planner_input = WarpPlannerInput::new(&input, &base, &origins, &context);

    let decisions = semantic_decisions(&planner_input, &WarpPlannerConfig::default());
    let decision = decisions
        .iter()
        .find(|decision| decision.post == ahead)
        .expect("ahead semantic decision");

    assert!(decision.admission.admissible, "{decisions:#?}");
    assert!(!decision.admission.rescue, "{decisions:#?}");
}
