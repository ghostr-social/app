use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, PlannerWatchEvidence, SemanticScore, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn learned_play_start_quantiles_replace_the_rank_fallback_deadline() {
    let input = snapshot(2, 20_000_000, 8_000, 0);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let current = input.candidates[0].post.clone();
    let ahead = input.candidates[1].post.clone();
    let learned = PlannerWatchEvidence::learned(4_200, 4_000, 8_000, 12_000, 2_500, None);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_watch(
            current.clone(),
            PlannerWatchEvidence::learned(10_000, 0, 0, 0, 10_000, Some(3_000)),
        )
        .with_watch(ahead.clone(), learned)
        .with_semantic(current, SemanticScore::Known(1_000_000))
        .with_semantic(ahead.clone(), SemanticScore::Known(420_000));

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let ladder = decision
        .generated
        .ladders
        .iter()
        .find(|item| item.post == ahead)
        .expect("ahead ladder");
    let deadlines = ladder
        .frontier
        .plans()
        .iter()
        .find(|plan| !plan.metrics.readiness_by_deadline.is_empty())
        .expect("retrieval deadline evidence")
        .metrics
        .readiness_by_deadline
        .iter()
        .map(|item| item.deadline_ms)
        .collect::<Vec<_>>();

    assert_eq!(deadlines, vec![4_000, 8_000, 12_000]);
    assert_eq!(learned.reach_probability_bps(), Some(4_200));
    assert_eq!(learned.probability_by_commitment_bps(), Some(2_500));
}
