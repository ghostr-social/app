use crate::adaptive::{AdaptivePlayabilityPolicy, DiscoveryDemand};
use crate::tests::adaptive_support::snapshot;

#[test]
fn playback_emergency_holds_discovery_expansion() {
    let input = snapshot(4, 20_000_000, 1_000, 2);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.discovery_demand, DiscoveryDemand::Hold);
}
