use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use std::time::Duration;

#[test]
fn material_latency_inflation_rejects_an_otherwise_fast_trial() {
    let mut policy = AdaptiveConcurrency::new(1, 2);
    drive(&mut policy, evidence(1, 1_000_000, 100), |value| {
        value.limit() == 2
    });
    drive(&mut policy, evidence(2, 1_400_000, 160), |value| {
        value.limit() == 1
    });
}

fn drive(
    policy: &mut AdaptiveConcurrency,
    evidence: ConcurrencyEvidence,
    done: impl Fn(&AdaptiveConcurrency) -> bool,
) {
    for _ in 0..20 {
        policy.observe(evidence);
        if done(policy) {
            return;
        }
    }
    panic!("policy did not reach the expected state");
}

fn evidence(active: usize, throughput: u64, ttfb_ms: u64) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: throughput,
        occupancy: ConcurrencyOccupancy::new(active, active),
        saturated: true,
        ttfb: Duration::from_millis(ttfb_ms),
        setback: NetworkSetback::None,
    }
}
