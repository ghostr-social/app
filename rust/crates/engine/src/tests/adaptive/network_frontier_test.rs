use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::tests::support::planned_playable_ms;
use crate::PostId;

#[test]
fn low_buffer_and_degraded_network_are_narrower_and_deeper_than_safe_capacity() {
    let policy = AdaptivePlayabilityPolicy;
    let mut poor = snapshot(12, 900_000, 1_000, 2);
    poor.network.connection_capacity = 1;
    let healthy = snapshot(12, 20_000_000, 20_000, 2);

    let poor_plan = policy.plan(&poor);
    let healthy_plan = policy.plan(&healthy);

    assert!(frontier(&poor_plan).len() <= 2, "{poor_plan:#?}");
    assert!(frontier(&healthy_plan).len() > frontier(&poor_plan).len());
    assert!(
        planned_playable_ms(&poor_plan, &PostId::new("p0"))
            > planned_playable_ms(&healthy_plan, &PostId::new("p0"))
    );
}
