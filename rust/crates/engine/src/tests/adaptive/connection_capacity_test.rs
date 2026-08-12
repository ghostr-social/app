use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, snapshot};

#[test]
fn measured_connection_capacity_expands_the_safe_frontier() {
    let policy = AdaptivePlayabilityPolicy;
    let mut serial = snapshot(16, 20_000_000, 20_000, 2);
    serial.network.connection_capacity = 1;
    let parallel = snapshot(16, 20_000_000, 20_000, 2);

    let serial = policy.plan(&serial);
    let parallel = policy.plan(&parallel);

    assert!(frontier(&parallel).len() > frontier(&serial).len());
}
