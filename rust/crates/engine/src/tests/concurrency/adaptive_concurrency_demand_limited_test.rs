use crate::concurrency::{AdaptiveConcurrency, ConcurrencyEvidence, NetworkSetback};
use std::time::Duration;

#[test]
fn quiet_or_partially_occupied_windows_do_not_change_capacity() {
    let mut policy = AdaptiveConcurrency::new(2, 4);
    for _ in 0..30 {
        policy.observe(ConcurrencyEvidence {
            aggregate_bytes_per_second: 100_000,
            active_transfers: 1,
            saturated: false,
            ttfb: Duration::from_secs(2),
            setback: NetworkSetback::None,
        });
    }

    assert_eq!(policy.limit(), 2);
    assert_eq!(policy.accepted_limit(), 2);
}
