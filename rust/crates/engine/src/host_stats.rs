//! Per-host performance model (plan §3): EWMA throughput, TTFB, and
//! failure ratio, updated by every probe and chunk transfer. Drives ETA
//! ranking and best-URL choice among imeta fallbacks. Pure and
//! deterministic — persistence lives in `host_stats_persistence`.

use crate::inventory_controller::Mode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// EWMA weight giving a half-life of ~10 samples: `1 - 0.5^(1/10)`.
/// After ten samples of a new steady value the estimate has moved
/// halfway from its old value to the new one.
pub const EWMA_ALPHA: f64 = 0.066_967_008_463_192_6;

/// Throughput assumed for hosts never transferred from, in bytes/s.
/// Deliberately optimistic (~4 MiB/s) so unknown hosts get sampled
/// instead of starved.
pub const OPTIMISTIC_THROUGHPUT_BPS: f64 = 4_194_304.0;

/// One exponentially weighted moving average; empty until first sample.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct Ewma {
    value: Option<f64>,
}

impl Ewma {
    fn observe(&mut self, sample: f64) {
        self.value = Some(match self.value {
            Some(previous) => EWMA_ALPHA * sample + (1.0 - EWMA_ALPHA) * previous,
            None => sample,
        });
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
struct HostRecord {
    throughput_bps: Ewma,
    ttfb_ms: Ewma,
    failure_ratio: Ewma,
}

/// Learned statistics for every host the engine has talked to.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HostStats {
    hosts: HashMap<String, HostRecord>,
}

impl HostStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a completed byte transfer. Zero-duration transfers carry
    /// no rate information and are ignored.
    pub fn record_transfer(&mut self, host: &str, bytes: u64, elapsed: Duration) {
        if elapsed.is_zero() {
            return;
        }
        let rate = bytes as f64 / elapsed.as_secs_f64();
        self.record(host).throughput_bps.observe(rate);
    }

    pub fn record_ttfb(&mut self, host: &str, ttfb_ms: u64) {
        self.record(host).ttfb_ms.observe(ttfb_ms as f64);
    }

    pub fn record_success(&mut self, host: &str) {
        self.record(host).failure_ratio.observe(0.0);
    }

    pub fn record_failure(&mut self, host: &str) {
        self.record(host).failure_ratio.observe(1.0);
    }

    /// Expected throughput in bytes/s; optimistic for unknown hosts.
    pub fn expected_throughput(&self, host: &str) -> f64 {
        self.lookup(host)
            .and_then(|record| record.throughput_bps.value)
            .unwrap_or(OPTIMISTIC_THROUGHPUT_BPS)
    }

    /// Expected time-to-first-byte; `None` until one is observed.
    pub fn expected_ttfb_ms(&self, host: &str) -> Option<f64> {
        self.lookup(host).and_then(|record| record.ttfb_ms.value)
    }

    /// Share of recent attempts that failed, in `[0, 1]`; `0` unknown.
    pub fn failure_ratio(&self, host: &str) -> f64 {
        self.lookup(host)
            .and_then(|record| record.failure_ratio.value)
            .unwrap_or(0.0)
    }

    /// Score multiplier for chunk scoring (plan §3). Comfort admits
    /// every host at full weight; hunger scales by measured speed
    /// (relative to the optimistic baseline, capped at 1) times
    /// reliability, so slow or failing hosts are skipped under pressure.
    pub fn host_factor(&self, host: &str, mode: Mode) -> f64 {
        match mode {
            Mode::Comfort => 1.0,
            Mode::Hunger => self.hunger_factor(host),
        }
    }

    /// Orders imeta URL candidates best-first for the downloader,
    /// replacing blind sequential fallback. Every mode discounts a
    /// candidate by its host's reliability, so a host that keeps
    /// failing cannot be picked over a healthy mirror however fast it
    /// once was (a host that only stumbles still outranks a slow one,
    /// which is how its stats heal); hunger additionally penalizes slow
    /// hosts. Stable: ties keep the imeta order; unparseable URLs sink.
    pub fn best_source(&self, urls: &[String], mode: Mode) -> Vec<String> {
        let mut ranked: Vec<(f64, &String)> = urls
            .iter()
            .map(|url| (self.source_score(url, mode), url))
            .collect();
        ranked.sort_by(|left, right| right.0.total_cmp(&left.0));
        ranked.into_iter().map(|(_, url)| url.clone()).collect()
    }

    /// Serializes the snapshot to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("host stats always serialize")
    }

    /// Restores a snapshot produced by [`Self::to_json`].
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    fn hunger_factor(&self, host: &str) -> f64 {
        self.speed_factor(host) * self.reliability(host)
    }

    /// Measured speed relative to the optimistic baseline, capped at 1.
    fn speed_factor(&self, host: &str) -> f64 {
        (self.expected_throughput(host) / OPTIMISTIC_THROUGHPUT_BPS).min(1.0)
    }

    /// Share of recent attempts that succeeded, in `[0, 1]`.
    fn reliability(&self, host: &str) -> f64 {
        1.0 - self.failure_ratio(host)
    }

    /// Expected throughput discounted by reliability, and in hunger by
    /// measured speed as well. Reliability counts once in either mode.
    fn source_score(&self, url: &str, mode: Mode) -> f64 {
        let Some(host) = host_of(url) else { return 0.0 };
        let expected = self.expected_throughput(&host) * self.reliability(&host);
        match mode {
            Mode::Comfort => expected,
            Mode::Hunger => expected * self.speed_factor(&host),
        }
    }

    fn lookup(&self, host: &str) -> Option<&HostRecord> {
        self.hosts.get(host)
    }

    fn record(&mut self, host: &str) -> &mut HostRecord {
        self.hosts.entry(host.to_owned()).or_default()
    }
}

/// Extracts the authority (host, keeping any port, lowercased) from a
/// URL. Recorders must key their stat updates with this same function
/// so `best_source` and the downloader agree on host identity.
pub fn host_of(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let authority = after_scheme
        .split(&['/', '?', '#'][..])
        .next()
        .unwrap_or_default();
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}
