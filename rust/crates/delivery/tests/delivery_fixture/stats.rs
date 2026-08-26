use core::time::Duration;
use ghostr_engine::host_stats::HostStats;
use std::path::Path;

pub async fn wait_for(path: &Path, ready: impl Fn(&HostStats) -> bool) -> HostStats {
    tokio::time::timeout(Duration::from_secs(2), async {
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
    .expect("host stats evidence")
}
