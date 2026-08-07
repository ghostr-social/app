//! Failure ratio: an EWMA over success (0) / failure (1) outcomes.

use crate::host_stats::{HostStats, EWMA_ALPHA};
use crate::tests::host_stats_support::assert_close;

const HOST: &str = "cdn.example";

/// `true` marks a failed attempt, `false` a successful one.
fn ratio_after(outcomes: &[bool]) -> f64 {
    let mut stats = HostStats::new();
    for failed in outcomes {
        if *failed {
            stats.record_failure(HOST);
        } else {
            stats.record_success(HOST);
        }
    }
    stats.failure_ratio(HOST)
}

#[test]
fn outcome_sequences_follow_the_ewma() {
    let cases: &[(&[bool], f64)] = &[
        (&[], 0.0),
        (&[true], 1.0),
        (&[false], 0.0),
        (&[true, false], 1.0 - EWMA_ALPHA),
        (&[false, true], EWMA_ALPHA),
        (&[false, false, true], EWMA_ALPHA),
        (&[true, true, false], 1.0 - EWMA_ALPHA),
    ];

    for (outcomes, expected) in cases {
        assert_close(ratio_after(outcomes), *expected);
    }
}

#[test]
fn ratios_from_different_hosts_stay_independent() {
    let mut stats = HostStats::new();

    stats.record_failure("flaky.example");
    stats.record_success("steady.example");

    assert_close(stats.failure_ratio("flaky.example"), 1.0);
    assert_close(stats.failure_ratio("steady.example"), 0.0);
}
