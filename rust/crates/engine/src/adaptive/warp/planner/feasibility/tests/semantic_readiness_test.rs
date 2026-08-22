use super::super::semantic_decisions;
use crate::adaptive::{
    AllocationPlan, PlannerCapability, PlannerContext, PlayerPreparation, SemanticScore,
    WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn exact_live_promotion_is_control_work_not_new_semantic_admission() {
    let post = PostId::new("p0");
    let node = crate::adaptive::ActionNode::new(
        1,
        post.clone(),
        crate::adaptive::ActionKind::Promote {
            active: crate::ActionId::new(7),
            maximum_bytes: 16,
        },
        crate::adaptive::ActionValue::default(),
    );
    let semantic = [crate::adaptive::SemanticDecision {
        post,
        admission: crate::adaptive::SemanticAdmission {
            admissible: false,
            rescue: false,
            rank_displacement: 0,
            censor: None,
        },
    }];

    assert!(super::super::semantically_admissible(&node, &semantic));
}

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
