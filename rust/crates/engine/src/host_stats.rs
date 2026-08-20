//! Per-host performance model (plan §3): EWMA throughput, TTFB, and
//! failure ratio, updated by every probe and chunk transfer. Drives ETA
//! ranking and best-URL choice among imeta fallbacks. Pure and
//! deterministic — persistence lives in `host_stats_persistence`.

use crate::origin_model::OriginModel;
use evidence::HostRecord;
pub use evidence::{ThroughputEstimate, ThroughputSample};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

mod evidence;
#[cfg(test)]
pub(crate) use evidence::EWMA_ALPHA;
mod retention;

/// Throughput assumed for hosts never transferred from, in bytes/s.
/// Deliberately optimistic (~4 MiB/s) so unknown hosts get sampled
/// instead of starved.
pub const OPTIMISTIC_THROUGHPUT_BPS: f64 = 4_194_304.0;

/// Learned statistics for every host the engine has talked to.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HostStats {
    #[serde(default)]
    overall: HostRecord,
    #[serde(default)]
    hosts: BTreeMap<String, HostRecord>,
    #[serde(default)]
    observation_sequence: u64,
    #[serde(default)]
    origin_model: OriginModel,
}

impl HostStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn origin_model(&self) -> &OriginModel {
        &self.origin_model
    }

    pub fn origin_model_mut(&mut self) -> &mut OriginModel {
        &mut self.origin_model
    }

    pub fn record_overall_throughput(&mut self, sample: ThroughputSample) -> bool {
        self.observation_sequence = self.observation_sequence.max(sample.observed_at_ms());
        self.overall.observe_throughput(sample)
    }

    pub fn record_host_throughput(&mut self, host: &str, sample: ThroughputSample) -> bool {
        self.observation_sequence = self.observation_sequence.max(sample.observed_at_ms());
        self.record(host).observe_throughput(sample)
    }

    pub fn overall_throughput(&self) -> Option<ThroughputEstimate> {
        self.overall.throughput()
    }

    pub fn host_throughput(&self, host: &str) -> Option<ThroughputEstimate> {
        self.lookup(host).and_then(HostRecord::throughput)
    }

    /// Records a completed byte transfer. Zero-duration transfers carry
    /// no rate information and are ignored.
    pub fn record_transfer(&mut self, host: &str, bytes: u64, elapsed: Duration) {
        if elapsed.is_zero() {
            return;
        }
        let elapsed_ms = elapsed.as_millis().min(u128::from(u64::MAX)).max(1) as u64;
        self.observation_sequence = self.observation_sequence.saturating_add(elapsed_ms);
        let sample = ThroughputSample::new(bytes, elapsed, self.observation_sequence, 1)
            .expect("positive transfer duration");
        self.record_overall_throughput(sample);
        self.record_host_throughput(host, sample);
    }

    pub fn record_ttfb(&mut self, host: &str, ttfb_ms: u64) {
        self.overall.observe_ttfb(ttfb_ms);
        self.record(host).observe_ttfb(ttfb_ms);
    }

    pub fn overall_ttfb(&self) -> Option<Duration> {
        self.overall.ttfb()
    }

    pub fn expected_ttfb(&self, host: &str) -> Option<Duration> {
        self.lookup(host)
            .and_then(HostRecord::ttfb)
            .or_else(|| self.overall_ttfb())
    }

    pub fn record_success(&mut self, host: &str) {
        self.record(host).observe_failure(0.0);
    }

    pub fn record_failure(&mut self, host: &str) {
        self.record(host).observe_failure(1.0);
    }

    /// Expected throughput in bytes/s; optimistic for unknown hosts.
    pub fn expected_throughput(&self, host: &str) -> f64 {
        self.host_throughput(host)
            .or_else(|| self.overall_throughput())
            .map(ThroughputEstimate::bytes_per_second)
            .unwrap_or(OPTIMISTIC_THROUGHPUT_BPS)
    }

    /// Share of recent attempts that failed, in `[0, 1]`; `0` unknown.
    pub fn failure_ratio(&self, host: &str) -> f64 {
        self.lookup(host)
            .and_then(HostRecord::failure_ratio)
            .unwrap_or(0.0)
    }

    /// Orders imeta URL candidates best-first for the downloader,
    /// replacing blind sequential fallback. Expected throughput is
    /// discounted by reliability; ties keep imeta order and invalid
    /// URLs sink.
    pub fn best_source(&self, urls: &[String]) -> Vec<String> {
        let mut ranked: Vec<(f64, &String)> = urls
            .iter()
            .map(|url| (self.source_score(url), url))
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
        let mut stats: Self = serde_json::from_str(json)?;
        stats.normalize_loaded();
        Ok(stats)
    }

    /// Share of recent attempts that succeeded, in `[0, 1]`.
    fn reliability(&self, host: &str) -> f64 {
        1.0 - self.failure_ratio(host)
    }

    fn source_score(&self, url: &str) -> f64 {
        let Some(host) = host_of(url) else { return 0.0 };
        self.expected_throughput(&host) * self.reliability(&host)
    }

    fn lookup(&self, host: &str) -> Option<&HostRecord> {
        self.hosts.get(host)
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
