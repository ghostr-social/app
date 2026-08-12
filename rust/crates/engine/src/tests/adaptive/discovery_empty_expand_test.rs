use crate::adaptive::{AdaptivePlayabilityPolicy, DiscoveryDemand};
use crate::tests::adaptive_support::snapshot;

#[test]
fn empty_candidate_supply_requests_discovery_expansion() {
    let input = snapshot(0, 20_000_000, 20_000, 2);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.discovery_demand, DiscoveryDemand::Expand);
}
