//! EWMA math: the first sample initializes an estimate, later samples
//! blend in with an alpha that gives a half-life of ten samples.

use crate::host_stats::{HostStats, EWMA_ALPHA, OPTIMISTIC_THROUGHPUT_BPS};
use crate::tests::host_stats_support::assert_close;
use std::time::Duration;

const HOST: &str = "cdn.example";

#[test]
fn alpha_encodes_a_ten_sample_half_life() {
    assert_close(EWMA_ALPHA, 1.0 - 0.5f64.powf(0.1));
}

#[test]
fn first_transfer_initializes_throughput() {
    let mut stats = HostStats::new();

    stats.record_transfer(HOST, 1_000_000, Duration::from_secs(2));

    assert_close(stats.expected_throughput(HOST), 500_000.0);
}

#[test]
fn second_transfer_blends_with_alpha() {
    let mut stats = HostStats::new();
    stats.record_transfer(HOST, 1_000_000, Duration::from_secs(1));

    stats.record_transfer(HOST, 2_000_000, Duration::from_secs(1));

    let expected = EWMA_ALPHA * 2_000_000.0 + (1.0 - EWMA_ALPHA) * 1_000_000.0;
    assert_close(stats.expected_throughput(HOST), expected);
}

#[test]
fn ten_opposite_samples_halve_the_distance() {
    let mut stats = HostStats::new();
    stats.record_transfer(HOST, 100, Duration::from_secs(1));

    for _ in 0..10 {
        stats.record_transfer(HOST, 0, Duration::from_secs(1));
    }

    assert_close(stats.expected_throughput(HOST), 50.0);
}

#[test]
fn ttfb_blends_like_throughput() {
    let mut stats = HostStats::new();

    stats.record_ttfb(HOST, 200);
    stats.record_ttfb(HOST, 100);

    let expected = EWMA_ALPHA * 100.0 + (1.0 - EWMA_ALPHA) * 200.0;
    assert_close(stats.expected_ttfb_ms(HOST).expect("recorded"), expected);
}

#[test]
fn zero_elapsed_transfers_carry_no_rate_and_are_ignored() {
    let mut stats = HostStats::new();

    stats.record_transfer(HOST, 1_000, Duration::ZERO);

    assert_close(stats.expected_throughput(HOST), OPTIMISTIC_THROUGHPUT_BPS);
}
