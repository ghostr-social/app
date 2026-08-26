use super::PlanInputs;
use crate::manager::concurrency::{planning_connection_capacity, HlsDemand};
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AllocationPlan, PlayabilitySnapshot, RequestOccupancy, SoftRequestCommitment,
};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[cfg(test)]
#[path = "request_capacity/hls_hard_budget_test.rs"]
mod hls_hard_budget_test;

#[derive(Clone, Copy)]
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

struct SoftCapacity {
    commitments: Vec<SoftRequestCommitment>,
    posts: usize,
}

pub(super) fn resolve(query: Query<'_>) -> RequestCapacity {
    let ordinary = query.snapshot.network.connection_capacity;
    let hls = planning_connection_capacity(
        ordinary,
        HlsDemand::new(
            hls_demand(query.inputs),
            query.inputs.hls_demand_expansion_allowed,
        ),
        query.snapshot.network.connection_ceiling,
        query.inputs.packet_loss_bps,
    );
    let soft = with_hedge_capacity(soft_capacity(&query), query.hedge_soft);
    let requested = query
        .occupancy
        .total()
        .saturating_add(soft.posts)
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

fn with_hedge_capacity(
    mut capacity: SoftCapacity,
    hedges: &[SoftRequestCommitment],
) -> SoftCapacity {
    for hedge in hedges {
        if !capacity.commitments.contains(hedge) {
            capacity.commitments.push(hedge.clone());
            capacity.posts = capacity.posts.saturating_add(1);
        }
    }
    capacity
}

fn soft_capacity(query: &Query<'_>) -> SoftCapacity {
    let bodies: HashSet<_> = query
        .inputs
        .in_flight
        .iter()
        .map(|item| item.post())
        .collect();
    let heads: HashSet<_> = query
        .inputs
        .active_head_probes
        .iter()
        .map(|item| item.post())
        .collect();
    let posts: HashSet<_> = query
        .base
        .allocations
        .iter()
        .filter(|item| !bodies.contains(&item.post))
        .filter(|item| !heads.contains(&item.post) || current_head_companion(query, &item.post))
        .map(|item| item.post.clone())
        .collect();
    let commitments = query
        .base
        .allocations
        .iter()
        .filter(|item| posts.contains(&item.post))
        .map(|item| {
            SoftRequestCommitment::new(item.post.clone(), item.source.clone(), item.request)
        })
        .collect();
    SoftCapacity {
        commitments,
        posts: posts.len(),
    }
}

fn current_head_companion(query: &Query<'_>, post: &PostId) -> bool {
    post == &query.snapshot.playback.current
        && query.inputs.active_head_probes.iter().any(|identity| {
            identity.post() == post
                && query
                    .state
                    .catalog()
                    .transfer_identity(post, identity.source().as_str())
                    .as_ref()
                    == Some(identity)
        })
}

#[cfg(test)]
#[path = "request_capacity_axiom_test.rs"]
pub(crate) mod axiom_test_support;
