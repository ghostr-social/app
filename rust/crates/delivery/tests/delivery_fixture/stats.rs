use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use std::path::Path;

pub fn seed_overall_throughput(root: &Path, bytes_per_second: u64) {
    let mut stats = HostStats::new();
    let observed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    let sample = ghostr_engine::host_stats::ThroughputSample::new(
        bytes_per_second,
        core::time::Duration::from_secs(1),
        observed_at_ms,
        1,
    )
    .expect("positive throughput fixture");
    stats.record_overall_throughput(sample);
    std::fs::create_dir_all(root).expect("stats fixture directory");
    std::fs::write(root.join("host_stats.json"), stats.to_json()).expect("seeded host stats");
}

pub async fn wait_for(path: &Path, ready: impl Fn(&HostStats) -> bool) -> HostStats {
    wait_for_within(path, Duration::from_secs(2), "host stats evidence", ready).await
}

pub async fn wait_for_within(
    path: &Path,
    deadline: Duration,
    label: &'static str,
    ready: impl Fn(&HostStats) -> bool,
) -> HostStats {
    tokio::time::timeout(deadline, async {
        loop {
            if let Ok(json) = tokio::fs::read_to_string(path).await {
                // A stale corrupt snapshot is not readiness evidence.
                if let Ok(stats) = HostStats::from_json(&json) {
                    if ready(&stats) {
                        return stats;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect(label)
}
