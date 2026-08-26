use crate::manager::plan::PlannedTransfer;
use crate::manager::traffic::OverallTrafficWindow;
use crate::manager::DeliveryWorker;
use core::num::NonZeroUsize;
use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::concurrency::{ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback};
use ghostr_engine::PostId;
use std::collections::HashSet;

mod observed;
pub(crate) use observed::{observed_admitted_capacity, observed_claimed_requests};
mod demand;
pub(crate) use demand::HlsDemand;

const SEVERE_PACKET_LOSS_BPS: u16 = 5_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlannedCapacity {
    pub(crate) total: usize,
    pub(crate) foreground_goal: usize,
}

impl PlannedCapacity {
    pub(super) fn with_selected_hedge(
        mut self,
        active: usize,
        ceiling: usize,
        selected: bool,
    ) -> Self {
        if selected {
            self.total = self.total.max(active.saturating_add(1)).min(ceiling.max(1));
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestConcurrencyLimits {
    global: usize,
    per_authority: usize,
}

impl RequestConcurrencyLimits {
    pub(crate) fn resolve(
        configured_global: usize,
        configured_per_authority: Option<NonZeroUsize>,
        debug_per_authority: usize,
    ) -> Self {
        let global = configured_global.max(1);
        let inherited = global.saturating_sub(1).max(1);
        let configured = configured_per_authority.map_or(inherited, NonZeroUsize::get);
        let debug = match debug_per_authority {
            0 => global,
            value => value,
        };
        Self {
            global,
            per_authority: global.min(configured).min(debug),
        }
    }

    pub(crate) const fn global(self) -> usize {
        self.global
    }

    pub(crate) const fn per_authority(self) -> usize {
        self.per_authority
    }
}

pub(crate) fn capacity_evidence(
    window: OverallTrafficWindow,
    saturated: bool,
    fallback_ttfb: Duration,
    occupancy: ConcurrencyOccupancy,
) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: finite_rate(window.bytes_per_second()),
        occupancy,
        saturated,
        ttfb: window.latest_ttfb().unwrap_or(fallback_ttfb),
        setback: NetworkSetback::None,
    }
}

pub(crate) fn request_occupancy(
    window: OverallTrafficWindow,
    admitted_capacity: usize,
    claimed_requests: usize,
) -> ConcurrencyOccupancy {
    ConcurrencyOccupancy::new(window.peak_active_transfers(), admitted_capacity)
        .with_claimed_requests(claimed_requests)
}

pub(crate) fn network_profile_setback(packet_loss_bps: u16) -> Option<NetworkSetback> {
    if packet_loss_bps >= SEVERE_PACKET_LOSS_BPS {
        return Some(NetworkSetback::SevereLoss);
    }
    match packet_loss_bps {
        0 => None,
        _ => Some(NetworkSetback::Failure),
    }
}

pub(crate) fn planning_connection_capacity(
    adaptive: usize,
    hls_demand: HlsDemand,
    ceiling: usize,
    packet_loss_bps: u16,
) -> usize {
    let demanded = match network_profile_setback(packet_loss_bps) {
        Some(NetworkSetback::SevereLoss) => 1,
        _ => adaptive.max(hls_demand.effective()),
    };
    demanded.min(ceiling.max(1)).max(1)
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
    pub(super) fn note_network_class_change(&mut self) {
        self.note_network_setback(NetworkSetback::Failure);
        self.warp_planner.reset_adaptation();
    }

    pub(super) fn note_network_profile_change(&mut self) {
        let loss = self.ctx.network.profile().packet_loss_bps;
        if let Some(setback) = network_profile_setback(loss) {
            self.note_network_setback(setback);
        }
    }

    pub(super) fn observe_capacity(&mut self, window: OverallTrafficWindow) {
        let saturated = self
            .additional_request_slot_demand
            .unwrap_or_else(|| self.queue.wanted_len() > self.downloads.len());
        let fallback = self.keeper.stats().overall_ttfb().unwrap_or(Duration::ZERO);
        let admitted = observed_admitted_capacity(
            self.downloads.admitted_capacity(),
            self.concurrency_limit(),
            self.connection_ceiling(),
        );
        let claimed = observed_claimed_requests(self.downloads.len(), self.segmented.active_len());
        let occupancy = request_occupancy(window, admitted, claimed);
        self.concurrency
            .observe(capacity_evidence(window, saturated, fallback, occupancy));
    }

    pub(super) fn note_network_setback(&mut self, setback: NetworkSetback) {
        let admitted = observed_admitted_capacity(
            self.downloads.admitted_capacity(),
            self.concurrency_limit(),
            self.connection_ceiling(),
        );
        let claimed = observed_claimed_requests(self.downloads.len(), self.segmented.active_len());
        self.concurrency.observe(ConcurrencyEvidence {
            aggregate_bytes_per_second: 0,
            occupancy: ConcurrencyOccupancy::new(claimed, admitted),
            saturated: false,
            ttfb: Duration::ZERO,
            setback,
        });
    }

    pub(super) fn update_concurrency_ceiling(&mut self) {
        self.concurrency.set_maximum(self.state.concurrency());
    }

    pub(super) fn concurrency_limit(&self) -> usize {
        self.concurrency.limit()
    }
}

fn finite_rate(rate: f64) -> u64 {
    if !rate.is_finite() || rate <= 0.0 {
        return 0;
    }
    rate.min(u64::MAX as f64).round() as u64
}
