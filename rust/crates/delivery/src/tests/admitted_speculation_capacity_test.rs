use super::support::planned_transfer;
use crate::manager::concurrency::planned_capacity;
use ghostr_engine::adaptive::PreemptionAuthority;
use std::collections::HashSet;

#[test]
fn admitted_speculation_can_use_a_safe_connection_from_the_plan() {
    let transfers = [
        planned_transfer("current", "same", PreemptionAuthority::PlaybackCritical),
        planned_transfer("next", "same", PreemptionAuthority::Transition),
        planned_transfer("later", "same", PreemptionAuthority::Speculative),
    ];

    assert_eq!(planned_capacity(1, 3, &transfers, &HashSet::new()).total, 3);
}
