use crate::adaptive::{
    AdaptivePlayabilityPolicy, HardBudget, InFlightAction, PlannerContext, ResourceCost,
    WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange, PostId};

#[test]
fn same_authority_paths_share_the_hard_request_budget() {
    let request = ResourceCost::new(0, 0, 0, 1);
    let mut budget = HardBudget::new(ResourceCost::new(0, 0, 0, 2), 1);

    assert!(budget.consume(&request, "https://a.example/first"));
    assert!(!budget.allows(&request, "https://a.example/second"));
    assert!(budget.allows(&request, "https://b.example/second"));
}

#[test]
fn a_cancelling_body_holds_its_authority_until_terminal_ack() {
    let mut input = snapshot(3, 20_000_000, 20_000, 0);
    input.network.connection_capacity = 2;
    input.network.connection_ceiling = 1;
    set_source(&mut input, 0, "https://a.example/active.mp4");
    set_source(&mut input, 1, "https://a.example/next.mp4");
    set_source(&mut input, 2, "https://b.example/next.mp4");
    input.candidates[0].in_flight.push(InFlightAction::range(
        ActionId::new(1),
        ByteRange::new(0, 64_000),
        "https://a.example/active.mp4",
        20_000,
        true,
    ));
    input.candidates[0].in_flight[0].cancelling = true;
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert_origin_admission(&decision, &PostId::new("p1"), false);
    assert_origin_admission(&decision, &PostId::new("p2"), true);
}

fn set_source(input: &mut crate::adaptive::PlayabilitySnapshot, index: usize, source: &str) {
    input.candidates[index].origins[0].source = source.to_owned();
}

fn assert_origin_admission(
    decision: &crate::adaptive::WarpPlanningDecision,
    post: &PostId,
    expected: bool,
) {
    let ids: Vec<_> = decision
        .generated
        .actions
        .iter()
        .filter(|action| &action.node.post == post && action.node.resources.requests > 0)
        .map(|action| action.node.id)
        .collect();
    assert!(
        !ids.is_empty(),
        "fixture must generate network work for {post:?}"
    );
    assert_eq!(
        ids.iter()
            .any(|id| decision.admissible_action_ids.contains(id)),
        expected,
        "unexpected admission for {post:?}: {decision:#?}"
    );
}
