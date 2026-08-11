use crate::host_stats::{HostStats, ThroughputSample};
use crate::playback::{AdaptiveBufferPolicy, MediaConsumption, NetworkConditions};
use std::time::Duration;

#[test]
fn fresh_repeated_evidence_needs_less_reserve_than_the_same_stale_estimate() {
    let mut stats = HostStats::new();
    for observed_at_ms in 1..=8 {
        let sample =
            ThroughputSample::new(2_000_000, Duration::from_secs(1), observed_at_ms * 1_000, 2)
                .unwrap();
        stats.record_host_throughput("cdn.example", sample);
    }
    let estimate = stats.host_throughput("cdn.example").unwrap();
    let ttfb = Duration::from_millis(100);
    let fresh = NetworkConditions::from_estimate(estimate, ttfb, 8_000);
    let stale = NetworkConditions::from_estimate(estimate, ttfb, 608_000);
    let media = MediaConsumption::new(4_000_000, 1_000);
    let policy = AdaptiveBufferPolicy::default();

    assert!(policy.target(fresh, media).steady() < policy.target(stale, media).steady());
}
