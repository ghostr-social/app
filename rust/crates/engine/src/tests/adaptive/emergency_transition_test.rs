use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, snapshot};
use crate::tests::support::planned_playable_ms;
use crate::PostId;

#[test]
fn low_current_buffer_still_prepares_the_likely_transition_when_capacity_remains() {
    let input = snapshot(4, 20_000_000, 1_000, 2);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(frontier(&plan).contains(&PostId::new("p0")));
    assert!(frontier(&plan).contains(&PostId::new("p1")));
    assert!(planned_playable_ms(&plan, &PostId::new("p0")) > 2_000);
}
