use crate::host_stats::{HostStats, ThroughputSample};
use std::time::Duration;

#[test]
fn persisted_host_evidence_is_bounded_by_recency() {
    let mut stats = HostStats::new();
    for index in 0..=HostStats::host_capacity() {
        let host = format!("cdn-{index:03}.example");
        stats.record_host_throughput(&host, sample(index as u64 + 1, index as u64 + 1));
    }

    let restored = HostStats::from_json(&stats.to_json()).unwrap();

    assert_eq!(restored.retained_host_count(), HostStats::host_capacity());
    assert!(restored.host_throughput("cdn-000.example").is_none());
    assert!(restored
        .host_throughput(&format!("cdn-{:03}.example", HostStats::host_capacity()))
        .is_some());
}

#[test]
fn legacy_snapshots_remain_readable() {
    let json = r#"{"hosts":{"cdn.example":{"throughput_bps":1000.0,"ttfb_ms":120.0,"failure_ratio":0.0}}}"#;

    let restored = HostStats::from_json(json).unwrap();

    assert_eq!(restored.expected_throughput("cdn.example"), 1_000.0);
    assert_eq!(
        restored
            .host_throughput("cdn.example")
            .unwrap()
            .sample_count(),
        1
    );
}

#[test]
fn oversized_legacy_snapshots_are_pruned_deterministically() {
    let hosts = (0..=HostStats::host_capacity())
        .map(|index| format!(r#""cdn-{index:03}.example":{{}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let restored = HostStats::from_json(&format!(r#"{{"hosts":{{{hosts}}}}}"#)).unwrap();

    assert_eq!(restored.retained_host_count(), HostStats::host_capacity());
    assert!(!restored.to_json().contains("cdn-000.example"));
    assert!(restored.to_json().contains("cdn-128.example"));
}

fn sample(rate: u64, observed_at_ms: u64) -> ThroughputSample {
    ThroughputSample::new(rate, Duration::from_secs(1), observed_at_ms, 1).unwrap()
}
