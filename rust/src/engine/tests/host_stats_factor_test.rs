//! Hunger mode penalizes slow and failing hosts; comfort admits every
//! host at full weight (plan §3).

use crate::engine::host_stats::{HostStats, EWMA_ALPHA, OPTIMISTIC_THROUGHPUT_BPS};
use crate::engine::inventory_controller::Mode;
use crate::engine::tests::host_stats_support::assert_close;
use std::time::Duration;

const HOST: &str = "cdn.example";

fn with_throughput(bytes_per_second: u64) -> HostStats {
    let mut stats = HostStats::new();
    stats.record_transfer(HOST, bytes_per_second, Duration::from_secs(1));
    stats
}

#[test]
fn comfort_factor_is_always_one() {
    let mut stats = with_throughput(1_000);
    stats.record_failure(HOST);

    assert_close(stats.host_factor(HOST, Mode::Comfort), 1.0);
}

#[test]
fn hunger_scales_by_speed_relative_to_the_optimistic_baseline() {
    let stats = with_throughput(OPTIMISTIC_THROUGHPUT_BPS as u64 / 2);

    assert_close(stats.host_factor(HOST, Mode::Hunger), 0.5);
}

#[test]
fn hunger_never_boosts_fast_hosts_above_one() {
    let stats = with_throughput(OPTIMISTIC_THROUGHPUT_BPS as u64 * 2);

    assert_close(stats.host_factor(HOST, Mode::Hunger), 1.0);
}

#[test]
fn hunger_zeroes_out_a_fully_failing_host() {
    let mut stats = HostStats::new();
    stats.record_failure(HOST);

    assert_close(stats.host_factor(HOST, Mode::Hunger), 0.0);
}

#[test]
fn hunger_compounds_speed_and_reliability() {
    let mut stats = with_throughput(OPTIMISTIC_THROUGHPUT_BPS as u64 / 2);
    stats.record_success(HOST);
    stats.record_failure(HOST);

    let expected = 0.5 * (1.0 - EWMA_ALPHA);
    assert_close(stats.host_factor(HOST, Mode::Hunger), expected);
}
