//! Host-statistics bookkeeping for the manager: transfer and probe
//! outcomes feed the single owned [`HostStats`]; the JSON snapshot is
//! persisted to the cache directory on a debounce.

use crate::manager::traffic::{OverallTrafficWindow, TrafficBatch, TrafficMeter};
use crate::manager::transfers::{InternalEvent, ProbeObservation};
use crate::manager::DeliveryWorker;
use crate::probe::media::is_usefulness_timeout;
use core::time::Duration;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_net::media_request_executor::MediaRequestAdmissionTimeout;
use log::warn;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

#[cfg(test)]
#[path = "stats/admission_timeout_test.rs"]
mod admission_timeout_test;
mod chunk;
mod hls;
#[cfg(test)]
#[path = "stats/no_request_start_test.rs"]
mod no_request_start_test;
#[cfg(test)]
#[path = "stats/open_body_persistence_test.rs"]
mod open_body_persistence_test;
mod origin;

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
        let origin = Instant::now();
        let origin_unix_ms = unix_time_ms();
        Self {
            stats,
            path,
            debounce,
            dirty: false,
            save_pending: false,
            traffic: TrafficMeter::new(origin, origin_unix_ms),
        }
    }

    pub fn stats(&self) -> &HostStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut HostStats {
        &mut self.stats
    }

    pub(crate) fn mark_origin_model_changed(&mut self) {
        self.dirty = true;
    }

    pub fn note_traffic(&mut self, batch: TrafficBatch) -> Option<OverallTrafficWindow> {
        self.dirty = true;
        self.traffic.apply(batch, &mut self.stats)
    }

    /// Mirrors the probe service's recording rules on the owned stats.
    pub fn note_probe(&mut self, done: &ProbeObservation) {
        if is_local_probe_timeout(&done.outcome) || done.attempt_context.is_none() {
            return;
        }
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
        let observed_at_ms = done
            .outcome
            .as_ref()
            .map_or_else(|_| unix_time_ms(), |result| result.observed.observed_at_ms);
        if let Some(observation) = origin::probe(done, observed_at_ms) {
            self.stats.origin_model_mut().observe(&observation);
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
            let _ = events.send(InternalEvent::Maintenance(
                crate::manager::transfers::MaintenanceEvent::SaveStats,
            ));
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

fn is_local_probe_timeout<T>(outcome: &anyhow::Result<T>) -> bool {
    is_admission_timeout(outcome)
        || outcome
            .as_ref()
            .is_err_and(|error| is_usefulness_timeout(error))
}

fn is_admission_timeout<T>(outcome: &anyhow::Result<T>) -> bool {
    outcome
        .as_ref()
        .is_err_and(|error| error.is::<MediaRequestAdmissionTimeout>())
}

impl DeliveryWorker {
    pub(super) fn absorb_traffic(&mut self) {
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

/// Loads persisted host stats. A missing or corrupt file yields fresh
/// stats — the model is a heuristic cache, never worth failing over.
pub(crate) async fn load_host_stats(path: &Path) -> HostStats {
    match tokio::fs::read_to_string(path).await {
        Ok(json) => HostStats::from_json(&json).unwrap_or_default(),
        Err(_) => HostStats::new(),
    }
}

/// Stages then renames the current snapshot so readers never observe a
/// truncated JSON document; callers decide the cadence.
pub(crate) async fn save_host_stats(path: &Path, stats: &HostStats) -> io::Result<()> {
    let staging = path.with_extension("json.tmp");
    if let Err(error) = tokio::fs::write(&staging, stats.to_json()).await {
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
        Err(error) => warn!("Host stats staging cleanup failed: {error}"),
    }
}
