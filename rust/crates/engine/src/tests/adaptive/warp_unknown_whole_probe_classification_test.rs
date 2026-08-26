use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate;
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, AllocationReason, HeadProbeHistory, PlannerCommand,
    PlannerContext, PlannerQuality, RetrievalRung, WarpPlanner, WarpPlannerInput,
    WholeBodyExhaustion,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn capped_unknown_whole_is_valued_as_discovery_not_completion() {
    let candidate = range_blind_candidate();
    let post = candidate.post.clone();
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_quality(
            &post,
            PlannerQuality::Estimated {
                expected_micros: 900_000,
                lower_micros: 800_000,
                uncertainty_bps: 1_000,
            },
        )
        .with_whole_body_exhaustion(
            &post,
            WholeBodyExhaustion::new(
                crate::adaptive::REQUEST_SLICE_BYTES,
                crate::adaptive::REQUEST_SLICE_BYTES + 1,
            )
            .expect("valid test fixture"),
        )
        .with_head_probe_history(&post, HeadProbeHistory::Completed);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let action = decision
        .generated
        .actions
        .iter()
        .find(|item| matches!(item.node.kind, ActionKind::FetchWhole { .. }))
        .expect("unknown whole probe");

    assert!(action.node.value.information_value_micros > 0);
    assert_eq!(action.node.value.delay_loss_micros, 0);
    assert_eq!(action.node.value.reserve_gain_micros, 0);
    assert_eq!(action.node.value.cache_gain_micros, 0);
    assert!(matches!(
        &action.command,
        PlannerCommand::Transfer(allocation)
            if allocation.reason == AllocationReason::MediaLayoutDiscovery
                && allocation.expected_playable_gain_ms == 0
                && allocation.utility.additional_playable_ms == 0
                && allocation.utility.score == 0.0
    ));
    let plan = decision
        .generated
        .ladders
        .iter()
        .flat_map(|ladder| ladder.frontier.plans())
        .find(|plan| {
            plan.actions
                .iter()
                .any(|kind| matches!(kind, ActionKind::FetchWhole { .. }))
        })
        .expect("probe remains in the retrieval ladder");
    assert_eq!(plan.terminal, RetrievalRung::Metadata);
    assert_eq!(plan.metrics.ready_playback_ms, 0);
    assert_eq!(plan.metrics.ready_coverage_ms, 0);
    assert_eq!(
        plan.metrics.size.lower,
        Some(crate::adaptive::REQUEST_SLICE_BYTES + 1)
    );
    assert_eq!(plan.metrics.size.upper, None);
    assert!(plan
        .metrics
        .readiness_by_deadline
        .iter()
        .all(|readiness| readiness.probability_bps == 0));
    assert_eq!(plan.metrics.quality.expected_micros, 0);
    assert_eq!(plan.metrics.quality.lower_micros, 0);
    assert!(plan.metrics.information_value_micros > 0);
}
