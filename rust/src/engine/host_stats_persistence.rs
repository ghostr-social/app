//! Thin async persistence for [`HostStats`] snapshots in the cache
//! directory. All statistics logic stays pure in `host_stats`; this
//! file only moves JSON to and from disk.

use crate::engine::host_stats::HostStats;
use std::io;
use std::path::Path;

/// Loads persisted host stats. A missing or corrupt file yields fresh
/// stats — the model is a heuristic cache, never worth failing over.
pub async fn load_host_stats(path: &Path) -> HostStats {
    match tokio::fs::read_to_string(path).await {
        Ok(json) => HostStats::from_json(&json).unwrap_or_default(),
        Err(_) => HostStats::new(),
    }
}

/// Writes the current snapshot; callers decide the cadence.
pub async fn save_host_stats(path: &Path, stats: &HostStats) -> io::Result<()> {
    tokio::fs::write(path, stats.to_json()).await
}
