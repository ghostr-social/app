use super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AllocationPlan, PlayabilitySnapshot, RequestOccupancy, SoftRequestCommitment,
};
use ghostr_engine::PostId;
use std::collections::HashSet;

pub(super) struct Query<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) snapshot: &'a PlayabilitySnapshot,
    pub(super) base: &'a AllocationPlan,
    pub(super) inputs: &'a PlanInputs<'a>,
    pub(super) occupancy: &'a RequestOccupancy,
}

pub(super) struct RequestCapacity {
    pub(super) tokens: u16,
    pub(super) ordinary_tokens: u16,
    pub(super) soft: Vec<SoftRequestCommitment>,
}

struct SoftCapacity {
    commitments: Vec<SoftRequestCommitment>,
    posts: usize,
}

pub(super) fn resolve(query: Query<'_>) -> RequestCapacity {
    let ordinary = query.snapshot.network.connection_capacity;
    let soft = soft_capacity(&query);
    let requested = query
        .occupancy
        .total()
        .saturating_add(soft.posts)
        .max(ordinary);
    let tokens = requested
        .min(query.snapshot.network.connection_ceiling)
        .min(u16::MAX as usize) as u16;
    RequestCapacity {
        tokens,
        ordinary_tokens: ordinary.min(u16::MAX as usize) as u16,
        soft: soft.commitments,
    }
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
