use crate::manager::quality::network_for;
use crate::tests::adaptive_plan_fixture;
use core::time::Duration;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::playback::NetworkConditions;
use ghostr_engine::PostId;

#[test]
fn host_history_cannot_exceed_the_observed_shared_path() {
    let state = adaptive_plan_fixture::state();
    let mut stats = HostStats::new();
    for now in 1..=8 {
        stats.record_host_throughput(
            "media.example",
            ThroughputSample::new(1_000_000, Duration::from_secs(1), now, 1).expect("fixture"),
        );
        stats.record_overall_throughput(
            ThroughputSample::new(100_000, Duration::from_secs(1), now, 2).expect("fixture"),
        );
    }
    let expected = NetworkConditions::from_estimate(
        stats.overall_throughput().expect("fixture"),
        Duration::from_millis(250),
        8,
    );
    assert_eq!(
        network_for(&state, &stats, &PostId::new("p0"), 8),
        Some(expected)
    );
}
