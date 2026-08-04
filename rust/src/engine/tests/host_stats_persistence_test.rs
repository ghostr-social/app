//! Async load/save of host stats snapshots in the cache directory.

use crate::engine::host_stats::HostStats;
use crate::engine::host_stats_persistence::{load_host_stats, save_host_stats};
use std::path::PathBuf;
use std::time::Duration;

fn temp_path(name: &str) -> PathBuf {
    let file = format!("ghostr_host_stats_{}_{name}.json", std::process::id());
    std::env::temp_dir().join(file)
}

#[tokio::test]
async fn missing_file_loads_fresh_stats() {
    let loaded = load_host_stats(&temp_path("missing")).await;

    assert_eq!(loaded, HostStats::new());
}

#[tokio::test]
async fn corrupt_file_loads_fresh_stats() {
    let path = temp_path("corrupt");
    tokio::fs::write(&path, "not json").await.expect("write fixture");

    let loaded = load_host_stats(&path).await;

    let _ = tokio::fs::remove_file(&path).await;
    assert_eq!(loaded, HostStats::new());
}

#[tokio::test]
async fn save_then_load_round_trips() {
    let path = temp_path("roundtrip");
    let mut stats = HostStats::new();
    stats.record_transfer("cdn.example", 1_000_000, Duration::from_secs(1));

    save_host_stats(&path, &stats).await.expect("save");
    let loaded = load_host_stats(&path).await;

    let _ = tokio::fs::remove_file(&path).await;
    assert_eq!(loaded, stats);
}
