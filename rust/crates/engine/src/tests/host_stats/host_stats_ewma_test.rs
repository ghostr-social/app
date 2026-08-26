//! Throughput smoothing follows sustained network behavior without
//! snapping to one unusually fast or slow transfer.

use crate::host_stats::{HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use crate::tests::host_stats_support::assert_close;
use core::time::Duration;

const HOST: &str = "cdn.example";

#[test]
fn first_transfer_initializes_throughput() {
    let mut stats = HostStats::new();

    stats.record_transfer(HOST, 1_000_000, Duration::from_secs(2));

    assert_close(stats.expected_throughput(HOST), 500_000.0);
}

#[test]
fn a_faster_transfer_moves_the_estimate_without_replacing_it() {
    let mut stats = HostStats::new();
    stats.record_transfer(HOST, 1_000_000, Duration::from_secs(1));

    stats.record_transfer(HOST, 2_000_000, Duration::from_secs(1));

    let estimate = stats.expected_throughput(HOST);
    assert!(estimate > 1_000_000.0);
    assert!(estimate < 2_000_000.0);
}

#[test]
fn sustained_slow_traffic_meaningfully_lowers_the_estimate() {
    let mut stats = HostStats::new();
    stats.record_transfer(HOST, 100, Duration::from_secs(1));

    for _ in 0..5 {
        stats.record_transfer(HOST, 0, Duration::from_secs(1));
    }

    let estimate = stats.expected_throughput(HOST);
    assert!(estimate > 25.0);
    assert!(estimate < 75.0);
}

#[test]
fn zero_elapsed_transfers_carry_no_rate_and_are_ignored() {
    let mut stats = HostStats::new();

    stats.record_transfer(HOST, 1_000, Duration::ZERO);

    assert_close(stats.expected_throughput(HOST), OPTIMISTIC_THROUGHPUT_BPS);
}
