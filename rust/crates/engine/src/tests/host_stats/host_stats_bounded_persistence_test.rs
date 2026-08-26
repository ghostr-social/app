use crate::host_stats::{HostStats, ThroughputSample};
use core::time::Duration;
use serde_json::Value;

const HOST_CAPACITY: usize = 128;

#[test]
fn persisted_host_evidence_is_bounded_by_recency() {
    let mut stats = HostStats::new();
    for index in 0..=HOST_CAPACITY {
        let host = format!("cdn-{index:03}.example");
        stats.record_host_throughput(&host, sample(index as u64 + 1, index as u64 + 1));
    }

    let restored = HostStats::from_json(&stats.to_json()).expect("valid test fixture");

    assert_eq!(host_count(&restored), HOST_CAPACITY);
    assert!(restored.host_throughput("cdn-000.example").is_none());
    assert!(restored
        .host_throughput(&format!("cdn-{HOST_CAPACITY:03}.example"))
        .is_some());
}

#[test]
fn legacy_snapshots_remain_readable() {
    let json = r#"{"hosts":{"cdn.example":{"throughput_bps":1000.0,"ttfb_ms":120.0,"failure_ratio":0.0}}}"#;

    let restored = HostStats::from_json(json).expect("valid test fixture");

    assert_eq!(restored.expected_throughput("cdn.example"), 1_000.0);
    assert_eq!(
        restored
            .host_throughput("cdn.example")
            .expect("valid test fixture")
            .sample_count(),
        1
    );
}

#[test]
fn oversized_legacy_snapshots_are_pruned_deterministically() {
    let hosts = (0..=HOST_CAPACITY)
        .map(|index| format!(r#""cdn-{index:03}.example":{{}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let restored =
        HostStats::from_json(&format!(r#"{{"hosts":{{{hosts}}}}}"#)).expect("valid test fixture");

    assert_eq!(host_count(&restored), HOST_CAPACITY);
    assert!(!restored.to_json().contains("cdn-000.example"));
    assert!(restored.to_json().contains("cdn-128.example"));
}

fn sample(rate: u64, observed_at_ms: u64) -> ThroughputSample {
    ThroughputSample::new(rate, Duration::from_secs(1), observed_at_ms, 1)
        .expect("valid test fixture")
}

fn host_count(stats: &HostStats) -> usize {
    serde_json::from_str::<Value>(&stats.to_json()).expect("valid test fixture")["hosts"]
        .as_object()
        .expect("valid test fixture")
        .len()
}
