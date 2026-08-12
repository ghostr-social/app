use ghostr_engine::host_stats::HostStats;
use std::path::Path;
use std::time::Duration;

pub async fn wait_for(path: &Path, ready: impl Fn(&HostStats) -> bool) -> HostStats {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Ok(json) = tokio::fs::read_to_string(path).await {
                // The snapshot is written non-atomically; a read that lands
                // mid-write sees a torn file. Treat it as not ready, like
                // the production loader does.
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
