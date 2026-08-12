use super::TransferKey;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Default)]
pub(super) struct ActiveTiming {
    active: HashMap<TransferKey, String>,
    overall_started: Option<Instant>,
    overall_elapsed: Duration,
    host_started: HashMap<String, Instant>,
    host_elapsed: HashMap<String, Duration>,
}

impl ActiveTiming {
    pub(super) fn open(&mut self, transfer: TransferKey, host: String, at: Instant) -> bool {
        if self.active.contains_key(&transfer) {
            return false;
        }
        if self.active.is_empty() {
            self.overall_started = Some(at);
        }
        if self.host_active(&host) == 0 {
            self.host_started.insert(host.clone(), at);
        }
        self.active.insert(transfer, host);
        true
    }

    pub(super) fn close(&mut self, transfer: TransferKey, at: Instant) {
        let Some(host) = self.active.remove(&transfer) else {
            return;
        };
        if self.active.is_empty() {
            finish(&mut self.overall_elapsed, &mut self.overall_started, at);
        }
        if self.host_active(&host) == 0 {
            finish_host(&host, &mut self.host_elapsed, &mut self.host_started, at);
        }
    }

    pub(super) fn host(&self, transfer: TransferKey) -> Option<&str> {
        self.active.get(&transfer).map(String::as_str)
    }

    pub(super) fn active(&self) -> usize {
        self.active.len()
    }

    pub(super) fn active_hosts(&self) -> Vec<String> {
        self.host_started.keys().cloned().collect()
    }

    pub(super) fn host_active(&self, host: &str) -> usize {
        self.active
            .values()
            .filter(|active| active.as_str() == host)
            .count()
    }

    pub(super) fn overall_elapsed(&self, at: Instant) -> Duration {
        self.overall_elapsed + running(self.overall_started, at)
    }

    pub(super) fn host_elapsed(&self, host: &str, at: Instant) -> Duration {
        self.host_elapsed.get(host).copied().unwrap_or_default()
            + running(self.host_started.get(host).copied(), at)
    }

    pub(super) fn reset(&mut self, at: Instant) {
        self.overall_elapsed = Duration::ZERO;
        self.overall_started = (!self.active.is_empty()).then_some(at);
        self.host_elapsed.clear();
        self.host_started.clear();
        for host in self.active.values() {
            self.host_started.entry(host.clone()).or_insert(at);
        }
    }
}

fn finish(elapsed: &mut Duration, started: &mut Option<Instant>, at: Instant) {
    *elapsed += running(*started, at);
    *started = None;
}

fn finish_host(
    host: &str,
    elapsed: &mut HashMap<String, Duration>,
    started: &mut HashMap<String, Instant>,
    at: Instant,
) {
    let duration = running(started.remove(host), at);
    *elapsed.entry(host.to_owned()).or_default() += duration;
}

fn running(started: Option<Instant>, at: Instant) -> Duration {
    started.map_or(Duration::ZERO, |start| at.saturating_duration_since(start))
}
