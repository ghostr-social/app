use crate::adaptive::{AdaptivePlayabilityPolicy, DiscoveryDemand};
use crate::tests::adaptive_support::snapshot;

/// Starting the first discovered video is exactly when the feed needs
/// more candidates; an emergency plan must still ask discovery to
/// expand while the roster is thin.
#[test]
fn startup_emergency_with_a_thin_roster_expands_discovery() {
    let input = snapshot(1, 20_000_000, 0, 0);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.discovery_demand, DiscoveryDemand::Expand, "{plan:#?}");
}

/// The current video's own allocations are exempt from the speculative
/// budget, so they must not count against it when deciding whether
/// discovery may grow the roster.
#[test]
fn current_video_allocations_do_not_exhaust_the_discovery_budget() {
    let input = snapshot(1, 800_000, 0, 0);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.discovery_demand, DiscoveryDemand::Expand, "{plan:#?}");
}
