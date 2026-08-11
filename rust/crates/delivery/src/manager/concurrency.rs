use crate::manager::plan::PlannedTransfer;
use crate::manager::traffic::OverallTrafficWindow;
use crate::manager::DeliveryWorker;
use ghostr_engine::concurrency::{ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback};
use ghostr_engine::tiers::Tier;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedCapacity {
    pub(crate) total: usize,
    pub(crate) foreground_goal: usize,
}

pub(crate) fn capacity_evidence(
    window: OverallTrafficWindow,
    saturated: bool,
    fallback_ttfb: Duration,
    admitted_capacity: usize,
) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: finite_rate(window.bytes_per_second()),
        occupancy: ConcurrencyOccupancy::new(window.peak_active_transfers(), admitted_capacity),
        saturated,
        ttfb: window.latest_ttfb().unwrap_or(fallback_ttfb),
        setback: NetworkSetback::None,
    }
}

pub(crate) fn effective_capacity(
    base: usize,
    ceiling: usize,
    foreground: bool,
    protected: bool,
) -> usize {
    let extra = usize::from(foreground && protected);
    base.saturating_add(extra).min(ceiling.max(1))
}

pub(crate) fn planned_capacity(
    base: usize,
    ceiling: usize,
    transfers: &[PlannedTransfer],
) -> PlannedCapacity {
    let foreground = transfers
        .iter()
        .filter(|work| {
            matches!(
                work.request.tier,
                Tier::T0PlaybackEmergency | Tier::T1CurrentTail
            )
        })
        .count();
    let protected = transfers
        .iter()
        .any(|work| work.request.tier == Tier::T2Startability);
    let total = effective_capacity(base, ceiling, foreground > 0, protected);
    PlannedCapacity {
        total,
        foreground_goal: foreground.min(base).min(total),
    }
}

impl DeliveryWorker {
    pub(crate) fn observe_capacity(&mut self, window: OverallTrafficWindow) {
        let saturated = self.queue.wanted_len() > self.downloads.len();
        let fallback = self.keeper.stats().overall_ttfb().unwrap_or(Duration::ZERO);
        let admitted = self.downloads.admitted_capacity();
        self.concurrency
            .observe(capacity_evidence(window, saturated, fallback, admitted));
    }

    pub(crate) fn note_network_setback(&mut self, setback: NetworkSetback) {
        self.concurrency.observe(ConcurrencyEvidence {
            aggregate_bytes_per_second: 0,
            occupancy: ConcurrencyOccupancy::new(
                self.downloads.len(),
                self.downloads.admitted_capacity(),
            ),
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
