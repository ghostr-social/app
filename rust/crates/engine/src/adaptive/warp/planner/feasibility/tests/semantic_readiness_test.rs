use super::super::semantic_decisions;
use crate::adaptive::{
    AllocationPlan, PlannerCapability, PlannerContext, PlayerPreparation, SemanticScore,
    WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn unsupported_cached_media_cannot_block_a_playable_rescue() {
    let mut input = snapshot(2, 20_000_000, 8_000, 0);
    input.candidates[0].finalized = true;
    input.candidates[0].player_preparation = PlayerPreparation::Unverified;
    input.candidates[1].player_preparation = PlayerPreparation::Unverified;
    let unsupported = input.candidates[0].post.clone();
    let rescue = input.candidates[1].post.clone();
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_capability(
            unsupported.clone(),
            PlannerCapability::reported(false, None, 1),
        )
        .with_semantic(unsupported, SemanticScore::Known(1_000))
        .with_semantic(rescue.clone(), SemanticScore::Known(1));
    let base = AllocationPlan::default();
    let origins = OriginModel::default();
    let planner_input = WarpPlannerInput::new(&input, &base, &origins, &context);
    let config = WarpPlannerConfig {
        semantic_top_k: 1,
        ..WarpPlannerConfig::default()
    };
    let decisions = semantic_decisions(&planner_input, &config);
    let rescued = decisions
        .iter()
        .find(|item| item.post == rescue)
        .expect("rescue candidate decision");

    assert!(rescued.admission.admissible);
    assert!(rescued.admission.rescue);
}
