use core::time::Duration;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use std::collections::BTreeMap;
use tokio::time::Instant;

mod event;
pub(crate) use event::{TrafficBatch, TrafficEvent, TransferKey};
mod mailbox;
pub(crate) use mailbox::{channel, TrafficInbox, TrafficPublisher};
mod timing;
use timing::ActiveTiming;
mod window;
pub(crate) use window::{OverallTrafficWindow, TrafficWindow};

pub(crate) const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct TrafficMeter {
    origin: Instant,
    origin_unix_ms: u64,
    timing: ActiveTiming,
    bytes: u64,
    host_bytes: BTreeMap<String, u64>,
    peak_active: usize,
    host_peak: BTreeMap<String, usize>,
    latest_ttfb: Option<Duration>,
}

impl TrafficMeter {
    pub(crate) fn new(origin: Instant, origin_unix_ms: u64) -> Self {
        Self {
            origin,
            origin_unix_ms,
            timing: ActiveTiming::default(),
            bytes: 0,
            host_bytes: BTreeMap::new(),
            peak_active: 0,
            host_peak: BTreeMap::new(),
            latest_ttfb: None,
        }
    }

    pub(crate) fn observe(&mut self, event: TrafficEvent, stats: &mut HostStats) {
        match event {
            TrafficEvent::Opened {
                transfer,
                host,
                ttfb,
                at,
            } => {
                if self.timing.open(transfer, host.clone(), at) {
                    self.note_open(&host, ttfb, stats);
                }
            }
            TrafficEvent::Resumed { transfer, host, at } => {
                if self.timing.open(transfer, host.clone(), at) { self.observe_concurrency(&host); }
            }
            TrafficEvent::Progress {
                transfer, bytes, ..
            } => self.progress(transfer, bytes),
            TrafficEvent::Closed { transfer, at } => self.timing.close(transfer, at),
        }
    }

    pub(crate) fn apply(
        &mut self,
        mut batch: TrafficBatch,
        stats: &mut HostStats,
    ) -> Option<OverallTrafficWindow> {
        batch.events_mut().sort_by_key(TrafficEvent::at);
        let window = batch.window();
        for event in batch.into_events() {
            self.observe(event, stats);
        }
        self.flush(window, stats)
    }

    pub(crate) fn flush(
        &mut self,
        window: TrafficWindow,
        stats: &mut HostStats,
    ) -> Option<OverallTrafficWindow> {
        let elapsed = self.active_elapsed(window);
        let summary = self.summary(elapsed, window.ended());
        if summary.is_some() {
            let sample = self.sample(self.bytes, elapsed, self.peak_active, window.ended());
            stats.record_overall_throughput(sample);
            self.flush_hosts(window, stats);
            self.reset_window(window.ended());
        } else if self.timing.active() == 0 {
            self.reset_window(window.ended());
        }
        summary
    }

    fn note_open(&mut self, host: &str, ttfb: Duration, stats: &mut HostStats) {
        stats.record_ttfb(host, ttfb.as_millis().min(u128::from(u64::MAX)) as u64);
        self.latest_ttfb = Some(ttfb);
        self.observe_concurrency(host);
    }

    fn progress(&mut self, transfer: TransferKey, bytes: u64) {
        let Some(host) = self.timing.host(transfer).map(str::to_owned) else {
            return;
        };
        self.bytes = self.bytes.saturating_add(bytes);
        let entry = self.host_bytes.entry(host).or_default();
        *entry = entry.saturating_add(bytes);
    }

    fn observe_concurrency(&mut self, host: &str) {
        self.peak_active = self.peak_active.max(self.timing.active());
        let active = self.timing.host_active(host);
        let peak = self.host_peak.entry(host.to_owned()).or_default();
        *peak = (*peak).max(active);
    }

    fn flush_hosts(&mut self, window: TrafficWindow, stats: &mut HostStats) {
        for (host, active) in core::mem::take(&mut self.host_peak) {
            let bytes = self.host_bytes.remove(&host).unwrap_or_default();
            let elapsed = self.host_elapsed(&host, window);
            if self.should_sample(bytes, elapsed) {
                let sample = self.sample(bytes, elapsed, active, window.ended());
                stats.record_host_throughput(&host, sample);
            }
        }
        self.host_bytes.clear();
    }

    fn sample(
        &self,
        bytes: u64,
        elapsed: Duration,
        active: usize,
        at: Instant,
    ) -> ThroughputSample {
        let observed = self.observed_at_ms(at);
        ThroughputSample::new(bytes, elapsed, observed, active)
            .expect("sample guard requires elapsed time and active transfers")
    }

    fn summary(&self, elapsed: Duration, at: Instant) -> Option<OverallTrafficWindow> {
        self.should_sample(self.bytes, elapsed).then(|| {
            OverallTrafficWindow::new(
                self.bytes,
                elapsed,
                self.peak_active,
                self.observed_at_ms(at),
                self.latest_ttfb,
            )
        })
    }

    fn active_elapsed(&self, window: TrafficWindow) -> Duration {
        self.timing.overall_elapsed(window.ended())
    }

    fn host_elapsed(&self, host: &str, window: TrafficWindow) -> Duration {
        self.timing.host_elapsed(host, window.ended())
    }

    fn should_sample(&self, bytes: u64, elapsed: Duration) -> bool {
        self.peak_active > 0 && !elapsed.is_zero() && (bytes > 0 || elapsed >= SAMPLE_INTERVAL)
    }

    fn observed_at_ms(&self, at: Instant) -> u64 {
        let elapsed = at.saturating_duration_since(self.origin).as_millis();
        self.origin_unix_ms
            .saturating_add(elapsed.min(u128::from(u64::MAX)) as u64)
    }

    fn reset_window(&mut self, at: Instant) {
        self.bytes = 0;
        self.peak_active = self.timing.active();
        self.host_peak.clear();
        self.latest_ttfb = None;
        self.timing.reset(at);
        for host in self.timing.active_hosts() {
            self.observe_concurrency(&host);
        }
    }
}
