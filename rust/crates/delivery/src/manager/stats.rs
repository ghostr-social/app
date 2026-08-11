//! Host-statistics bookkeeping for the manager: transfer and probe
//! outcomes feed the single owned [`HostStats`]; the JSON snapshot is
//! persisted to the cache directory on a debounce.

use crate::manager::traffic::{OverallTrafficWindow, TrafficBatch, TrafficMeter};
use crate::manager::transfers::{ChunkDone, InternalEvent, ProbeDone};
use crate::manager::DeliveryWorker;
use ghostr_engine::host_stats::{host_of, HostStats};
use log::{trace, warn};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

pub(crate) struct StatsKeeper {
    stats: HostStats,
    path: PathBuf,
    debounce: Duration,
    dirty: bool,
    save_pending: bool,
    traffic: TrafficMeter,
}

impl StatsKeeper {
    pub async fn load(path: PathBuf, debounce: Duration) -> Self {
        let stats = load_host_stats(&path).await;
        Self {
            stats,
            path,
            debounce,
            dirty: false,
            save_pending: false,
            traffic: TrafficMeter::new(Instant::now(), unix_time_ms()),
        }
    }

    pub fn stats(&self) -> &HostStats {
        &self.stats
    }

    /// Mirrors the downloader's recording rules on the owned stats.
    pub fn note_chunk(&mut self, done: &ChunkDone) {
        let Some(host) = host_of(&done.url) else {
            return;
        };
        match &done.outcome {
            Ok(_) => {
                self.stats.record_success(&host);
            }
            Err(_) => self.stats.record_failure(&host),
        }
        self.dirty = true;
    }

    pub fn note_traffic(&mut self, batch: TrafficBatch) -> Option<OverallTrafficWindow> {
        self.dirty = true;
        let window = self.traffic.apply(batch, &mut self.stats);
        if let Some(window) = window {
            trace_window(window);
        }
        window
    }

    /// Mirrors the probe service's recording rules on the owned stats.
    pub fn note_probe(&mut self, done: &ProbeDone) {
        let Some(host) = host_of(&done.url) else {
            return;
        };
        match &done.outcome {
            Ok(result) => {
                self.stats
                    .record_ttfb(&host, result.ttfb.as_millis() as u64);
                self.stats.record_success(&host);
            }
            Err(_) => self.stats.record_failure(&host),
        }
        self.dirty = true;
    }

    /// Schedules at most one debounced save; later changes ride along.
    pub fn schedule_save(&mut self, events: &UnboundedSender<InternalEvent>) {
        if !self.dirty || self.save_pending {
            return;
        }
        self.save_pending = true;
        let events = events.clone();
        let debounce = self.debounce;
        tokio::spawn(async move {
            tokio::time::sleep(debounce).await;
            let _ = events.send(InternalEvent::SaveStats);
        });
    }

    /// Persists the snapshot now; a failed write stays dirty so the
    /// next event schedules another attempt.
    pub async fn save_now(&mut self) {
        self.save_pending = false;
        if !self.dirty {
            return;
        }
        match save_host_stats(&self.path, &self.stats).await {
            Ok(()) => self.dirty = false,
            Err(error) => warn!("Host stats snapshot failed: {error}"),
        }
    }
}

impl DeliveryWorker {
    pub(crate) fn absorb_traffic(&mut self) {
        let batch = self.traffic.drain(Instant::now());
        if let Some(window) = self.keeper.note_traffic(batch) {
            self.observe_capacity(window);
        }
    }
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

fn trace_window(window: OverallTrafficWindow) {
    trace!(
        "traffic window: bytes={}, elapsed={:?}, rate={}, peak={}, at={}, ttfb={:?}",
        window.bytes(),
        window.elapsed(),
        window.bytes_per_second(),
        window.peak_active_transfers(),
        window.observed_at_ms(),
        window.latest_ttfb(),
    );
}

/// Loads persisted host stats. A missing or corrupt file yields fresh
/// stats — the model is a heuristic cache, never worth failing over.
pub(crate) async fn load_host_stats(path: &Path) -> HostStats {
    match tokio::fs::read_to_string(path).await {
        Ok(json) => HostStats::from_json(&json).unwrap_or_default(),
        Err(_) => HostStats::new(),
    }
}

/// Writes the current snapshot; callers decide the cadence.
pub(crate) async fn save_host_stats(path: &Path, stats: &HostStats) -> io::Result<()> {
    tokio::fs::write(path, stats.to_json()).await
}
