//! Host-statistics bookkeeping for the manager: transfer and probe
//! outcomes feed the single owned [`HostStats`]; the JSON snapshot is
//! persisted to the cache directory on a debounce.

use crate::engine::host_stats::{host_of, HostStats};
use crate::engine::host_stats_persistence::{load_host_stats, save_host_stats};
use crate::video::delivery_transfers::{ChunkDone, InternalEvent, ProbeDone};
use log::warn;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub(crate) struct StatsKeeper {
    stats: HostStats,
    path: PathBuf,
    debounce: Duration,
    dirty: bool,
    save_pending: bool,
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
            Ok(result) => {
                if result.bytes_written > 0 {
                    self.stats
                        .record_transfer(&host, result.bytes_written, done.elapsed);
                }
                self.stats.record_success(&host);
            }
            Err(_) => self.stats.record_failure(&host),
        }
        self.dirty = true;
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
