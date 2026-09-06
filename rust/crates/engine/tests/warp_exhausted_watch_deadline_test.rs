use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, PlannerWatchEvidence, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn an_immediate_future_deadline_cannot_be_replaced_by_rank_based_slack() {
    let input = snapshot(2, 20_000_000, 8_000, 0);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let current = input.candidates[0].post.clone();
    let ahead = input.candidates[1].post.clone();
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_watch(
            &current,
            PlannerWatchEvidence::learned(10_000, 0, 0, 0, 10_000, Some(3_000)),
        )
        .with_watch(
            &ahead,
            PlannerWatchEvidence::learned(4_200, 0, 0, 0, 10_000, None),
        );
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    assert_eq!(
        deadlines(&decision, &ahead),
        vec![0],
        "exhausted watch means the next video may be needed immediately"
    );
    assert_eq!(
        deadlines(&decision, &current),
        vec![3_000],
        "current playback retains its explicit safety deadline"
    );
}

fn deadlines(decision: &crate::adaptive::WarpPlanningDecision, post: &crate::PostId) -> Vec<u64> {
    decision
        .generated
        .ladders
        .iter()
        .find(|ladder| &ladder.post == post)
        .expect("candidate ladder")
        .frontier
        .plans()
        .iter()
        .find(|plan| !plan.metrics.readiness_by_deadline.is_empty())
        .expect("readiness evidence")
        .metrics
        .readiness_by_deadline
        .iter()
        .map(|value| value.deadline_ms)
        .collect()
}
