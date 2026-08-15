use super::QoeStats;
use log::warn;
use std::io;
use std::path::Path;

pub async fn load_qoe_stats(path: &Path) -> QoeStats {
    match tokio::fs::read_to_string(path).await {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => QoeStats::default(),
    }
}

pub async fn save_qoe_stats(path: &Path, stats: &QoeStats) -> io::Result<()> {
    let staging = path.with_extension("json.tmp");
    let body = serde_json::to_vec(stats).expect("QoE aggregates always serialize");
    if let Err(error) = tokio::fs::write(&staging, body).await {
        remove_staging(&staging).await;
        return Err(error);
    }
    if let Err(error) = tokio::fs::rename(&staging, path).await {
        remove_staging(&staging).await;
        return Err(error);
    }
    Ok(())
}

async fn remove_staging(path: &Path) {
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => warn!("QoE staging cleanup failed: {error}"),
    }
}
