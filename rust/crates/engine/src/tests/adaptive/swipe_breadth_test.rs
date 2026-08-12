use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, planned_playable_ms, snapshot};
use crate::PostId;

#[test]
fn rapid_forward_navigation_trades_per_candidate_depth_for_more_breadth() {
    let policy = AdaptivePlayabilityPolicy;
    let slow = policy.plan(&snapshot(16, 20_000_000, 20_000, 2));
    let rapid = policy.plan(&snapshot(16, 20_000_000, 20_000, 30));

    assert!(frontier(&rapid).len() > frontier(&slow).len());
    assert!(
        planned_playable_ms(&rapid, &PostId::new("p1"))
            < planned_playable_ms(&slow, &PostId::new("p1"))
    );
}
