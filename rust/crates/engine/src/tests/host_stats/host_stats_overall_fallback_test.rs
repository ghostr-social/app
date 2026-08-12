use crate::host_stats::{HostStats, ThroughputSample};
use std::time::Duration;

#[test]
fn unknown_hosts_use_observed_overall_until_host_evidence_arrives() {
    let mut stats = HostStats::new();
    stats.record_overall_throughput(sample(2_000_000, 10, 2));

    assert_eq!(stats.expected_throughput("new.example"), 2_000_000.0);

    stats.record_host_throughput("slow.example", sample(250_000, 20, 1));

    assert_eq!(stats.expected_throughput("slow.example"), 250_000.0);
    assert_eq!(stats.expected_throughput("other.example"), 2_000_000.0);
    assert_eq!(
        stats.overall_throughput().unwrap().bytes_per_second(),
        2_000_000.0
    );
}

fn sample(rate: u64, observed_at_ms: u64, active: usize) -> ThroughputSample {
    ThroughputSample::new(rate, Duration::from_secs(1), observed_at_ms, active).unwrap()
}
