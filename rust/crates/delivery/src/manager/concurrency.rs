use crate::manager::plan::PlannedTransfer;
use crate::manager::traffic::OverallTrafficWindow;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::concurrency::{ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback};
use ghostr_engine::PostId;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::time::Duration;

const SEVERE_PACKET_LOSS_BPS: u16 = 5_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedCapacity {
    pub(crate) total: usize,
    pub(crate) foreground_goal: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestConcurrencyLimits {
    global: usize,
    per_authority: usize,
    segmented_compatibility: usize,
}

impl RequestConcurrencyLimits {
    pub(crate) fn resolve(
        configured_global: usize,
        configured_per_authority: Option<NonZeroUsize>,
        debug_per_authority: usize,
    ) -> Self {
        let global = configured_global.max(1);
        let configured = configured_per_authority.map_or(global, NonZeroUsize::get);
        let debug = match debug_per_authority {
            0 => global,
            value => value,
        };
        Self {
            global,
            per_authority: global.min(configured).min(debug),
            segmented_compatibility: global.min(debug),
        }
    }

    pub(crate) const fn global(self) -> usize {
        self.global
    }

    pub(crate) const fn per_authority(self) -> usize {
        self.per_authority
    }

    pub(crate) const fn segmented_compatibility(self) -> usize {
        self.segmented_compatibility
    }
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
