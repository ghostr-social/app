//! JSON snapshot round-trip keeps every learned host statistic.

use crate::host_stats::HostStats;
use core::time::Duration;

#[test]
fn round_trip_preserves_all_recorded_stats() {
    let mut stats = HostStats::new();
    stats.record_transfer("cdn.example", 2_000_000, Duration::from_secs(1));
    stats.record_ttfb("cdn.example", 120);
    stats.record_failure("flaky.example");
    stats.record_success("flaky.example");

    let restored = HostStats::from_json(&stats.to_json()).expect("valid json");

    assert_eq!(restored, stats);
}

#[test]
fn empty_stats_round_trip() {
    let stats = HostStats::new();

    let restored = HostStats::from_json(&stats.to_json()).expect("valid json");

    assert_eq!(restored, stats);
}

#[test]
fn corrupt_json_is_rejected() {
    assert!(HostStats::from_json("not json").is_err());
}
