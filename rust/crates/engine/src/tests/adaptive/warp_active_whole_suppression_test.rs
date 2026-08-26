use super::warp_range_noncompliant_unknown_size_generation_test::range_blind_candidate;
use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    ActionKind, ActivePlannerContext, AdaptivePlayabilityPolicy, HeadProbeHistory, HedgeInput,
    IdentityProof, InFlightAction, PlannerCapability, PlannerContext, PlannerLimits,
    RetrievalRequest, TransformCapability, TransformKind, WholeBodyContract, WholeBodyExhaustion,
    WholeFetchReason,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn changed_network_envelope_does_not_duplicate_an_active_whole_fetch() {
    let prior_cap = crate::adaptive::REQUEST_SLICE_BYTES;
    let active_cap = crate::adaptive::BOOTSTRAP_DIRECT_FETCH_BYTES;
    let mut candidate = range_blind_candidate();
    let post = candidate.post.clone();
    candidate.in_flight.push(active_whole(active_cap));
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates = vec![candidate];
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let hedge = HedgeInput::new(
        crate::ActionId::new(9),
        ActionKind::FetchWhole {
            maximum_bytes: active_cap,
        },
    )
    .with_timing(1_000, 900)
    .with_value(5_000, 1_000)
    .with_network_envelope(active_cap);
    let active = ActivePlannerContext::new(crate::ActionId::new(9), post.clone()).with_hedge(
        hedge,
        IdentityProof::VerifiedHash([3; 32]),
        "https://mirror.example/media",
    );
    let context = PlannerContext::explicitly_unavailable(&input)
        .with_head_probe_history(&post, HeadProbeHistory::Completed)
        .with_capability(
            &post,
            PlannerCapability::reported(
                false,
                Some(TransformCapability::new(
                    TransformKind::Remux,
                    300,
                    active_cap,
                )),
                1,
            ),
        )
        .with_whole_body_exhaustion(
            &post,
            WholeBodyExhaustion::new(prior_cap, prior_cap + 1).expect("valid test fixture"),
        )
        .with_limits(PlannerLimits {
            network_burst_bytes: active_cap * 4,
            network_rate_bytes_per_second: prior_cap,
            cpu_ms: 0,
            request_tokens: 2,
            per_origin_requests: 2,
        })
        .with_active(active);

    let generated = WarpActionGenerator::generate(&input, &base, &OriginModel::default(), &context);

    assert!(generated.actions.iter().all(|action| !matches!(
        action.node.kind,
        ActionKind::FetchWhole { .. } | ActionKind::Transform(_)
    )));
    assert!(generated
        .actions
        .iter()
        .any(|action| matches!(action.node.kind, ActionKind::Hedge { .. })));
}

fn active_whole(maximum_bytes: u64) -> InFlightAction {
    let request = RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped { maximum_bytes },
        reason: WholeFetchReason::DirectCrossover,
    };
    InFlightAction {
        action_id: crate::ActionId::new(9),
        request,
        effective_bytes: request.requested_bytes(),
        reserved_storage_bytes: maximum_bytes,
        source: "https://origin.example/media".to_owned(),
        committed_until_ms: 20_000,
        identity_current: true,
        cancelling: false,
    }
}
