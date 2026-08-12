use crate::adaptive::{AdaptivePlayabilityPolicy, DiscoveryDemand};
use crate::tests::adaptive_support::snapshot;

#[test]
fn safe_playback_with_spare_resources_requests_more_candidates() {
    let input = snapshot(1, 20_000_000, 20_000, 2);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.discovery_demand, DiscoveryDemand::Expand);
}
