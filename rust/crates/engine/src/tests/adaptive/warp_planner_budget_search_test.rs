use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, BeamConfig, HardBudget, NetworkTokenBucket, ResourceCost,
    ResourceObservation, SearchPruneReason, ShadowPriceController, WarpSearch,
};
use crate::{ByteRange, PostId, RequestAuthority};

#[path = "warp_search_audit_test.rs"]
mod search_audit_test;

fn node(id: u16, kind: ActionKind, score: i64, requires: &[u16]) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("video"),
        kind,
        ActionValue::from_net_micros(score),
    )
    .with_resources(ResourceCost::new(64_000, 64_000, 0, 1))
    .with_origin("https://origin.example/media")
    .requiring(requires)
}

#[test]
fn network_token_bucket_enforces_burst_and_refills_from_elapsed_time() {
    let mut bucket = NetworkTokenBucket::new(1_000, 500, 10_000);
    assert!(bucket.consume(900, 10_000));
    assert!(!bucket.consume(200, 10_000));
    assert_eq!(bucket.available(11_000), 600);
    assert!(bucket.consume(600, 11_000));
}

#[test]
fn beam_search_values_probe_then_fetch_and_commits_only_its_first_action() {
    let prefix = node(1, ActionKind::Prefix(ByteRange::new(0, 64_000)), 2, &[]);
    let range = node(
        2,
        ActionKind::FetchRange(ByteRange::new(64_000, 128_000)),
        20,
        &[1],
    );
    let whole = node(
        3,
        ActionKind::FetchWhole {
            maximum_bytes: 300_000,
        },
        15,
        &[],
    );
    let budget = HardBudget::new(ResourceCost::new(500_000, 500_000, 10, 3), 2);

    let selected = WarpSearch::new(BeamConfig::new(3, 16, 128, 10_000))
        .choose_first(&[whole, range, prefix.clone()], budget);

    assert_eq!(selected.action, Some(prefix));
    assert_eq!(selected.committed_actions, 1);
    assert!(!selected.used_greedy_fallback);
    assert_eq!(selected.chosen_plan.unwrap().action_ids, vec![1, 2]);
    assert!(selected
        .pruned_plans
        .iter()
        .any(|plan| plan.reason == SearchPruneReason::MutuallyExclusive));
}

#[test]
fn planner_latency_budget_uses_greedy_positive_fallback() {
    let probe = node(1, ActionKind::Prefix(ByteRange::new(0, 64_000)), 2, &[]);
    let whole = node(
        2,
        ActionKind::FetchWhole {
            maximum_bytes: 300_000,
        },
        15,
        &[],
    );
    let selected = WarpSearch::new(BeamConfig::new(3, 16, 128, 0))
        .choose_first(&[probe, whole.clone()], HardBudget::unlimited());
    assert_eq!(selected.action, Some(whole));
    assert!(selected.used_greedy_fallback);
}

#[test]
fn hard_tokens_reject_over_budget_work_and_shadow_prices_follow_pressure() {
    let mut budget = HardBudget::new(ResourceCost::new(100, 100, 10, 2), 1);
    let authority = RequestAuthority::from_url("https://a.example/media").unwrap();
    assert!(budget.consume(&ResourceCost::new(80, 50, 5, 1), Some(&authority)));
    assert!(!budget.consume(&ResourceCost::new(30, 10, 1, 1), Some(&authority)));
    let mut prices = ShadowPriceController::default();
    prices.observe(
        ResourceObservation::new(200, 80, 8, 4),
        ResourceObservation::new(100, 100, 10, 2),
    );
    let raised = prices.prices();
    prices.observe(
        ResourceObservation::new(0, 0, 0, 0),
        ResourceObservation::new(100, 100, 10, 2),
    );
    assert!(raised.network_micros > 0 && raised.request_micros > 0);
    assert!(prices.prices().network_micros < raised.network_micros);
}
