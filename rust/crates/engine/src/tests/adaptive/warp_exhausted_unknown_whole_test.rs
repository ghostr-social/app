use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate;
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, HeadProbeHistory, PlannerContext, PlannerLimits,
    WarpActionGenerator, WarpPlanner, WarpPlannerInput, WholeBodyExhaustion,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn exhausted_unknown_whole_cap_advances_to_the_next_geometric_probe() {
    let decision = decision(
        crate::adaptive::REQUEST_SLICE_BYTES,
        crate::adaptive::REQUEST_SLICE_BYTES + 1,
        crate::adaptive::REQUEST_SLICE_BYTES * 4,
    );
    let action = whole_action(&decision, crate::adaptive::REQUEST_SLICE_BYTES * 4);
    assert!(decision.admissible_action_ids.contains(&action.node.id));
}

#[test]
fn exhausted_cap_is_deferred_until_the_network_envelope_can_make_progress() {
    let maximum = crate::adaptive::REQUEST_SLICE_BYTES * 4;
    let observed = maximum + 1;
    let deferred = decision(maximum, observed, maximum);
    let action = whole_action(&deferred, maximum * 4);
    assert!(!deferred.admissible_action_ids.contains(&action.node.id));

    let recovered_burst = observed + crate::adaptive::REQUEST_SLICE_BYTES;
    let recovered = decision(maximum, observed, recovered_burst);
    let action = whole_action(&recovered, recovered_burst);
    assert!(recovered.admissible_action_ids.contains(&action.node.id));
}

fn decision(maximum: u64, observed: u64, burst: u64) -> crate::adaptive::WarpPlanningDecision {
    let candidate = range_blind_candidate();
    let post = candidate.post.clone();
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(post.clone(), HeadProbeHistory::Completed)
        .with_whole_body_exhaustion(post, WholeBodyExhaustion::new(maximum, observed).unwrap())
        .with_limits(PlannerLimits {
            network_burst_bytes: burst,
            network_rate_bytes_per_second: 0,
            cpu_ms: 0,
            request_tokens: 2,
            per_origin_requests: 2,
        });

    WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ))
}

fn whole_action(
    decision: &crate::adaptive::WarpPlanningDecision,
    expected: u64,
) -> &crate::adaptive::GeneratedAction {
    decision
        .generated
        .actions
        .iter()
        .find(|action| matches!(action.node.kind, ActionKind::FetchWhole { maximum_bytes } if maximum_bytes == expected))
        .expect("whole-body transition")
}

#[test]
fn complete_session_bytes_do_not_generate_another_whole_fetch() {
    let total = crate::adaptive::REQUEST_SLICE_BYTES + 1;
    let mut candidate =
        super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate_with_size(
            Some(total),
        );
    candidate.present = vec![crate::ByteRange::new(0, total)];
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);

    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);

    assert!(generated
        .actions
        .iter()
        .all(|action| !matches!(action.node.kind, ActionKind::FetchWhole { .. })));
}
