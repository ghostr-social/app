use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, HeadProbeHistory, OriginHealth, PlannerCommand, PlannerContext,
    RetrievalRequest, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

const PREFERRED: &str = "https://fast.example/video.mp4";

#[test]
fn unknown_whole_uses_the_same_preferred_source_as_its_exhaustion_evidence() {
    let mut candidate = range_blind_candidate();
    candidate.preferred_source = Some(PREFERRED.into());
    candidate.origins.push(OriginHealth {
        source: PREFERRED.into(),
        available: true,
        throughput_bps: 40_000_000,
        rtt_ms: 20,
        packet_loss_bps: 0,
        failure_bps: 0,
    });
    let post = candidate.post.clone();
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(post, HeadProbeHistory::Completed);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let source = decision.generated.actions.iter().find_map(|action| {
        let PlannerCommand::Transfer(allocation) = &action.command else {
            return None;
        };
        matches!(allocation.request, RetrievalRequest::FetchWhole { .. })
            .then_some(allocation.source.as_str())
    });

    assert_eq!(source, Some(PREFERRED));
}
