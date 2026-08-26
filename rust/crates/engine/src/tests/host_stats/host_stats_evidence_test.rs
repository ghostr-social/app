use crate::host_stats::{HostStats, ThroughputSample};
use core::time::Duration;

#[test]
fn throughput_evidence_tracks_recency_and_variability() {
    let mut stats = HostStats::new();
    stats.record_host_throughput("cdn.example", sample(1_000, 10, 1));
    stats.record_host_throughput("cdn.example", sample(1_000, 20, 2));

    let stable = stats
        .host_throughput("cdn.example")
        .expect("valid test fixture");
    assert_eq!(stable.sample_count(), 2);
    assert_eq!(stable.last_observed_at_ms(), 20);
    assert_eq!(stable.variability_bytes_per_second(), 0.0);

    stats.record_host_throughput("cdn.example", sample(4_000, 30, 1));

    let changed = stats
        .host_throughput("cdn.example")
        .expect("valid test fixture");
    assert_eq!(changed.sample_count(), 3);
    assert!(changed.variability_bytes_per_second() > 0.0);
}

#[test]
fn invalid_or_stale_samples_do_not_change_evidence() {
    assert!(ThroughputSample::new(1, Duration::ZERO, 1, 1).is_none());
    assert!(ThroughputSample::new(1, Duration::from_secs(1), 1, 0).is_none());
    let mut stats = HostStats::new();
    assert!(stats.record_host_throughput("cdn.example", sample(1_000, 20, 1)));

    assert!(!stats.record_host_throughput("cdn.example", sample(4_000, 10, 4)));

    let estimate = stats
        .host_throughput("cdn.example")
        .expect("valid test fixture");
    assert_eq!(estimate.bytes_per_second(), 1_000.0);
    assert_eq!(estimate.sample_count(), 1);
    assert_eq!(estimate.last_observed_at_ms(), 20);
}

#[test]
fn moving_average_depends_on_elapsed_time_not_server_chunk_cadence() {
    let mut one_window = initialized_stats();
    let mut ten_windows = initialized_stats();

    one_window.record_host_throughput(
        "cdn.example",
        ThroughputSample::new(45_000, Duration::from_secs(5), 6_000, 1)
            .expect("valid test fixture"),
    );
    for step in 1..=10 {
        ten_windows.record_host_throughput(
            "cdn.example",
            ThroughputSample::new(4_500, Duration::from_millis(500), 1_000 + step * 500, 1)
                .expect("valid test fixture"),
        );
    }

    let once = one_window.expected_throughput("cdn.example");
    let often = ten_windows.expected_throughput("cdn.example");
    assert!((once - often).abs() < 1.0, "once={once}, often={often}");
}

#[test]
fn coalesced_samples_at_one_timestamp_still_use_measured_elapsed_time() {
    let mut stats = initialized_stats();
    let coalesced = ThroughputSample::new(10_000, Duration::from_secs(5), 1_000, 1)
        .expect("valid test fixture");

    stats.record_host_throughput("cdn.example", coalesced);

    let estimate = stats.expected_throughput("cdn.example");
    assert!(estimate > 1_000.0);
    assert!(estimate < 2_000.0);
}

fn initialized_stats() -> HostStats {
    let mut stats = HostStats::new();
    stats.record_host_throughput("cdn.example", sample(1_000, 1_000, 1));
    stats
}

fn sample(rate: u64, observed_at_ms: u64, active: usize) -> ThroughputSample {
    ThroughputSample::new(rate, Duration::from_secs(1), observed_at_ms, active)
        .expect("valid test fixture")
}
