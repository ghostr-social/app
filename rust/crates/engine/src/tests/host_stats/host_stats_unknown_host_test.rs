//! Unknown hosts are treated optimistically so new hosts get sampled
//! instead of starved.

use crate::host_stats::{HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use crate::tests::host_stats_support::assert_close;

const HOST: &str = "never-seen.example";

#[test]
fn unknown_host_gets_the_optimistic_throughput() {
    let stats = HostStats::new();

    assert_close(stats.expected_throughput(HOST), OPTIMISTIC_THROUGHPUT_BPS);
}

#[test]
fn unknown_host_has_no_failures() {
    let stats = HostStats::new();

    assert_close(stats.failure_ratio(HOST), 0.0);
}
