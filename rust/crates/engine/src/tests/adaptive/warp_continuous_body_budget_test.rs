use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate_with_size;
use crate::adaptive::{AdaptivePlayabilityPolicy, HeadProbeHistory, PlannerContext, PlannerLimits,
    WarpPlanner, WarpPlannerInput, REQUEST_SLICE_BYTES};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn continuous_body_reserves_one_network_window_and_its_full_storage_envelope() {
    let total = 16 * 1024 * 1024;
    let mut input = snapshot(1, 6_000_000, 0, 0);
    input.candidates = vec![range_blind_candidate_with_size(Some(total))];
    let post = input.candidates[0].post.clone();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(&post, HeadProbeHistory::Completed)
        .with_limits(PlannerLimits { network_burst_bytes: 1024 * 1024,
            network_rate_bytes_per_second: 750_000, cpu_ms: 0, request_tokens: 1, per_origin_requests: 1 });
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input, &base, &OriginModel::default(), &context,
    ));
    let selected = decision.selected.expect("a single continuous response can make bounded progress");
    assert_eq!(selected.node.resources.network_bytes, REQUEST_SLICE_BYTES);
    assert_eq!(selected.node.resources.storage_bytes, total);
    assert_eq!(selected.node.resources.requests, 1);
}
