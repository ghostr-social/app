use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use core::time::Duration;

#[test]
fn a_full_protected_override_can_supply_base_learning_evidence() {
    let mut policy = AdaptiveConcurrency::new(1, 3);
    let evidence = protected_override_evidence(2);

    for _ in 0..3 {
        assert_eq!(
            policy.observe(evidence),
            1,
            "trial must wait for four windows"
        );
    }

    assert_eq!(policy.observe(evidence), 2);
    for _ in 0..3 {
        assert_eq!(policy.observe(evidence), 2, "the trial remains active");
    }
    assert_eq!(policy.observe(evidence), 1, "an ungained trial is rejected");
}

#[test]
fn a_partially_occupied_override_cannot_supply_learning_evidence() {
    let mut policy = AdaptiveConcurrency::new(1, 3);

    for _ in 0..8 {
        assert_eq!(policy.observe(protected_override_evidence(1)), 1);
    }

    assert_eq!(policy.limit(), 1);
}

fn protected_override_evidence(active: usize) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: 2_000_000,
        occupancy: ConcurrencyOccupancy::new(active, 2),
        saturated: true,
        ttfb: Duration::from_millis(100),
        setback: NetworkSetback::None,
    }
}
