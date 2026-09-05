use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::tests::support::planned_playable_ms;
use crate::PostId;

#[test]
fn rapid_forward_navigation_reduces_depth_within_the_bounded_window() {
    let policy = AdaptivePlayabilityPolicy;
    let slow = policy.plan(&snapshot(16, 20_000_000, 20_000, 2));
    let rapid = policy.plan(&snapshot(16, 20_000_000, 20_000, 30));

    assert!(frontier(&rapid).len() <= 2);
    assert!(
        planned_playable_ms(&rapid, &PostId::new("p1"))
            < planned_playable_ms(&slow, &PostId::new("p1"))
    );
}
