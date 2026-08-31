use super::super::semantic_decisions;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, PlannerContext, PlayerPreparation, WarpPlannerConfig,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn reserve_rescue_does_not_broaden_ordinary_semantic_admission() {
    let mut input = snapshot(7, 2_500_000, 20_000, 120);
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    for candidate in &mut input.candidates[1..=4] {
        candidate.present = candidate
            .startup
            .as_ref()
            .expect("reserve startup")
            .ranges()
            .to_vec();
    }
    for candidate in &mut input.candidates[5..] {
        candidate.player_preparation = PlayerPreparation::Unverified;
    }
    let base = AdaptivePlayabilityPolicy.plan(&input);

    let decisions = decide(&input, &base, 5);
    let target = admission(&decisions, "p5");
    assert!(target.admissible && target.rescue);
    assert!(!admission(&decisions, "p6").admissible);

    let normal_target = admission(&decide(&input, &base, 6), "p5");
    assert!(normal_target.admissible && !normal_target.rescue);
    let normal = crate::adaptive::AllocationPlan {
        mode: ControlMode::Normal,
        ..base
    };
    assert!(!admission(&decide(&input, &normal, 5), "p5").admissible);
}

fn decide(
    input: &crate::adaptive::PlayabilitySnapshot,
    base: &crate::adaptive::AllocationPlan,
    top_k: usize,
) -> Vec<crate::adaptive::SemanticDecision> {
    let origins = OriginModel::default();
    let context = PlannerContext::explicitly_unavailable(input);
    let planner_input = WarpPlannerInput::new(input, base, &origins, &context);
    let config = WarpPlannerConfig {
        semantic_top_k: top_k,
        ..WarpPlannerConfig::default()
    };
    semantic_decisions(&planner_input, &config)
}

fn admission(
    decisions: &[crate::adaptive::SemanticDecision],
    post: &str,
) -> crate::adaptive::SemanticAdmission {
    decisions
        .iter()
        .find(|item| item.post == PostId::new(post))
        .expect("semantic decision")
        .admission
}
