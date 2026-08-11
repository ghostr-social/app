use crate::manager::traffic::OverallTrafficWindow;
use crate::manager::DeliveryWorker;
use ghostr_engine::concurrency::{ConcurrencyEvidence, NetworkSetback};
use std::time::Duration;

pub(crate) fn capacity_evidence(
    window: OverallTrafficWindow,
    saturated: bool,
    fallback_ttfb: Duration,
) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: finite_rate(window.bytes_per_second()),
        active_transfers: window.peak_active_transfers(),
        saturated,
        ttfb: window.latest_ttfb().unwrap_or(fallback_ttfb),
        setback: NetworkSetback::None,
    }
}

impl DeliveryWorker {
    pub(crate) fn observe_capacity(&mut self, window: OverallTrafficWindow) {
        let saturated = self.queue.wanted_len() > self.downloads.len();
        let fallback = self.keeper.stats().overall_ttfb().unwrap_or(Duration::ZERO);
        self.concurrency
            .observe(capacity_evidence(window, saturated, fallback));
    }

    pub(crate) fn note_network_setback(&mut self, setback: NetworkSetback) {
        self.concurrency.observe(ConcurrencyEvidence {
            aggregate_bytes_per_second: 0,
            active_transfers: self.downloads.len(),
            saturated: false,
            ttfb: Duration::ZERO,
            setback,
        });
    }

    pub(crate) fn update_concurrency_ceiling(&mut self) {
        self.concurrency.set_maximum(self.state.concurrency());
    }

    pub(crate) fn concurrency_limit(&self) -> usize {
        self.concurrency.limit()
    }
}

fn finite_rate(rate: f64) -> u64 {
    if !rate.is_finite() || rate <= 0.0 {
        return 0;
    }
    rate.min(u64::MAX as f64).round() as u64
}
