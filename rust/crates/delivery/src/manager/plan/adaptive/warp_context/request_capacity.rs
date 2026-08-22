use super::PlanInputs;
use crate::manager::concurrency::planning_connection_capacity;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AllocationPlan, PlayabilitySnapshot, RequestOccupancy, SoftRequestCommitment,
};

mod soft;

pub(super) struct Query<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) snapshot: &'a PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) inputs: &'a PlanInputs<'a>,
    pub(super) occupancy: &'a RequestOccupancy,
    pub(super) hedge_soft: &'a [SoftRequestCommitment],
}

pub(super) struct RequestCapacity {
    pub(super) tokens: u16,
    pub(super) ordinary_tokens: u16,
    pub(super) hls_tokens: u16,
    pub(super) soft: Vec<SoftRequestCommitment>,
}

pub(super) fn resolve(query: Query<'_>) -> RequestCapacity {
    let ordinary = query.snapshot.network.connection_capacity;
    let hls = planning_connection_capacity(
        ordinary,
        hls_demand(query.inputs),
        query.snapshot.network.connection_ceiling,
        query.inputs.packet_loss_bps,
    );
    let soft = soft::resolve(&query, query.hedge_soft);
    let guarded_requests = match query.occupancy.total() <= ordinary {
        true => soft.guarded_requests,
        false => 0,
    };
    let requested = query
        .occupancy
        .total()
        .saturating_add(soft.requests)
        .saturating_add(guarded_requests)
        .max(ordinary)
        .max(hls);
    let tokens = requested
        .min(query.snapshot.network.connection_ceiling)
        .min(u16::MAX as usize) as u16;
    RequestCapacity {
        tokens,
        ordinary_tokens: ordinary.min(u16::MAX as usize) as u16,
        hls_tokens: hls.min(u16::MAX as usize) as u16,
        soft: soft.commitments,
    }
}

pub(super) fn hls_burst_floor(snapshot: &PlayabilitySnapshot, hls_tokens: u16) -> u64 {
    let live = snapshot
        .hls_candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.state,
                ghostr_engine::adaptive::HlsBootstrapState::Pending { .. }
                    | ghostr_engine::adaptive::HlsBootstrapState::Active { .. }
            )
        })
        .count()
        .min(usize::from(hls_tokens));
    ghostr_engine::adaptive::HlsBootstrapStage::FirstSegment
        .maximum_bytes()
        .saturating_mul(live as u64)
}

fn hls_demand(inputs: &PlanInputs<'_>) -> usize {
    inputs
        .hls_candidates
        .iter()
        .filter(|candidate| {
            matches!(
                &candidate.state,
                ghostr_engine::adaptive::HlsBootstrapState::Pending { .. }
                    | ghostr_engine::adaptive::HlsBootstrapState::Active { .. }
            )
        })
        .count()
}
