use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, snapshot};

#[test]
fn measured_capacity_cannot_expand_the_frontier_past_the_core_window() {
    let policy = AdaptivePlayabilityPolicy;
    let mut serial = snapshot(16, 20_000_000, 20_000, 2);
    serial.network.connection_capacity = 1;
    let parallel = snapshot(16, 20_000_000, 20_000, 2);

    let serial = policy.plan(&serial);
    let parallel = policy.plan(&parallel);

    assert!(frontier(&parallel).len() >= frontier(&serial).len());
}
