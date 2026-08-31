use crate::adaptive::{
    AdaptivePlayabilityPolicy, HardBudget, InFlightAction, PlannerContext, ResourceCost,
    WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive::warp_planner_test_assertions::set_source;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange, PostId};

#[test]
fn request_authority_uses_scheme_normalized_host_and_effective_port() {
    for (active, candidate) in [
        ("https://EXAMPLE.com:443/a", "https://example.com/b"),
        ("https://[2001:0db8::1]/a", "https://[2001:db8::1]:443/b"),
        (
            "https://bücher.example/a",
            "https://xn--bcher-kva.example/b",
        ),
    ] {
        assert!(
            !candidate_admitted(active, candidate),
            "{active} vs {candidate}"
        );
    }
    assert!(candidate_admitted(
        "http://example.com/a",
        "https://example.com/b"
    ));
}

#[test]
fn malformed_or_credentialed_request_origins_are_not_admissible() {
    for source in [
        "not a URL",
        "ftp://example.com/media",
        "https://user:secret@example.com/media",
    ] {
        let mut input = snapshot(1, 20_000_000, 20_000, 0);
        set_source(&mut input, 0, source);
        let decision = decision(&input);
        assert!(decision.generated.actions.iter().all(|action| {
            action.node.post != PostId::new("p0") || action.node.resources.requests == 0
        }));
    }
}

#[test]
fn zero_request_local_work_does_not_require_an_authority() {
    let local = ResourceCost::new(0, 0, 1, 0);
    assert!(HardBudget::unlimited().consume(&local, None));
}

fn candidate_admitted(active: &str, candidate: &str) -> bool {
    let mut input = snapshot(2, 20_000_000, 20_000, 0);
    input.network.connection_capacity = 2;
    input.network.connection_ceiling = 2;
    input.network.per_authority_request_limit = 1;
    set_source(&mut input, 0, active);
    set_source(&mut input, 1, candidate);
    input.candidates[0].in_flight.push(InFlightAction::range(
        ActionId::new(1),
        ByteRange::new(0, 64_000),
        active,
        20_000,
        true,
    ));
    let decision = decision(&input);
    let post = PostId::new("p1");
    let ids: Vec<_> = decision
        .generated
        .actions
        .iter()
        .filter(|action| action.node.post == post && action.node.resources.requests > 0)
        .map(|action| action.node.id)
        .collect();
    !ids.is_empty()
        && ids
            .iter()
            .any(|id| decision.admissible_action_ids.contains(id))
}

fn decision(input: &crate::adaptive::PlayabilitySnapshot) -> crate::adaptive::WarpPlanningDecision {
    let base = AdaptivePlayabilityPolicy.plan(input);
    WarpPlanner::default().plan(WarpPlannerInput::new(
        input,
        &base,
        &OriginModel::default(),
        &PlannerContext::explicitly_unavailable(input),
    ))
}
