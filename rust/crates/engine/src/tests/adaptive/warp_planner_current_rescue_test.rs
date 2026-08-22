use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, FeedOffset, MediaLayout, PlannerContext,
    ViewProbability, WarpPlanner, WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::{ColdStartPrior, ColdStartSelector, OriginModel, RequestMethod};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn emergency_rescue_capacity_belongs_to_the_playback_current_post() {
    let input = current_second_snapshot();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    assert_ne!(base.mode, ControlMode::Normal, "{base:#?}");
    let context = PlannerContext::explicitly_unavailable(&input);
    let config = WarpPlannerConfig::default().with_rescue_thresholds(2_000, 2_000);
    let origins = reliable_origin();
    let decision =
        WarpPlanner::new(config).plan(WarpPlannerInput::new(&input, &base, &origins, &context));

    let protected: Vec<_> = decision
        .generated
        .actions
        .iter()
        .filter(|action| {
            decision
                .reserve
                .protected_action_ids
                .contains(&action.node.id)
        })
        .map(|action| action.node.post.clone())
        .collect();
    assert!(
        !decision.reserve.degraded,
        "reserve={:#?}\nactions={:#?}",
        decision.reserve, decision.generated.actions
    );
    assert!(!protected.is_empty(), "rescue action is required");
    assert!(protected.iter().all(|post| post == &input.playback.current));
}

fn current_second_snapshot() -> crate::adaptive::PlayabilitySnapshot {
    let mut input = snapshot(2, 20_000_000, 0, 20);
    input.commitment_ms = 1_000_000_000;
    input.playback.current = PostId::new("p1");
    input.candidates[0].feed_offset = FeedOffset::new(-1);
    input.candidates[0].view_probability = ViewProbability::new(0.1).unwrap();
    input.candidates[1].feed_offset = FeedOffset::new(0);
    input.candidates[1].view_probability = ViewProbability::new(1.0).unwrap();
    input
        .candidates
        .iter_mut()
        .for_each(|candidate| candidate.layout = MediaLayout::RequiresCompleteFile);
    input
}

fn reliable_origin() -> OriginModel {
    let mut origins = OriginModel::default();
    origins.register_cold_start(
        ColdStartSelector::default().with_method(RequestMethod::FullGet),
        ColdStartPrior::new(1_000_000_000.0, 0.01, 1, 100_000_000),
    );
    origins
}
