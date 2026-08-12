use crate::manager::plan::PlannedTransfer;
use crate::manager::traffic::OverallTrafficWindow;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::concurrency::{ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback};
use ghostr_engine::PostId;
use std::collections::HashSet;
use std::time::Duration;

const SEVERE_PACKET_LOSS_BPS: u16 = 5_000;

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

pub(crate) fn network_profile_setback(packet_loss_bps: u16) -> NetworkSetback {
    if packet_loss_bps >= SEVERE_PACKET_LOSS_BPS {
        return NetworkSetback::SevereLoss;
    }
    match packet_loss_bps {
        0 => NetworkSetback::None,
        _ => NetworkSetback::Failure,
    }
}

pub(crate) fn connection_ceiling(configured: usize, per_host: usize) -> usize {
    let configured = configured.max(1);
    match per_host {
        0 => configured,
        limit => configured.min(limit.max(1)),
    }
}

pub(crate) fn planned_capacity(
    base: usize,
    ceiling: usize,
    transfers: &[PlannedTransfer],
    retained: &HashSet<PostId>,
) -> PlannedCapacity {
    let foreground = transfers
        .iter()
        .filter(|work| work.request.authority == PreemptionAuthority::PlaybackCritical)
        .count();
    let admitted: HashSet<_> = transfers
        .iter()
        .map(|work| work.request.chunk.post.clone())
        .chain(retained.iter().cloned())
        .collect();
    let total = base.max(admitted.len()).max(1).min(ceiling.max(1));
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
