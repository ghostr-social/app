//! Unknown hosts are treated optimistically so new hosts get sampled
//! instead of starved.

use crate::engine::host_stats::{HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use crate::engine::inventory_controller::Mode;
use crate::engine::tests::host_stats_support::assert_close;

const HOST: &str = "never-seen.example";

#[test]
fn unknown_host_gets_the_optimistic_throughput() {
    let stats = HostStats::new();

    assert_close(stats.expected_throughput(HOST), OPTIMISTIC_THROUGHPUT_BPS);
}

#[test]
fn unknown_host_has_no_failures_and_no_ttfb() {
    let stats = HostStats::new();

    assert_close(stats.failure_ratio(HOST), 0.0);
    assert_eq!(stats.expected_ttfb_ms(HOST), None);
}

#[test]
fn unknown_host_factor_is_neutral_in_both_modes() {
    let stats = HostStats::new();

    assert_close(stats.host_factor(HOST, Mode::Hunger), 1.0);
    assert_close(stats.host_factor(HOST, Mode::Comfort), 1.0);
}
