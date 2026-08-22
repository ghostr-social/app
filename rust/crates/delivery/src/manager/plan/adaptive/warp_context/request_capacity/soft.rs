use super::Query;
use ghostr_engine::adaptive::{Allocation, ControlMode, SoftRequestCommitment};
use ghostr_engine::PostId;
use std::collections::HashSet;

pub(super) struct Capacity {
    pub(super) commitments: Vec<SoftRequestCommitment>,
    pub(super) requests: usize,
    pub(super) guarded_requests: usize,
}

pub(super) fn resolve(query: &Query<'_>, hedges: &[SoftRequestCommitment]) -> Capacity {
    let active = active_commitments(query);
    let mut capacity = ordinary(query);
    extend_emergency_current(query, &active, &mut capacity);
    for hedge in hedges {
        add(&mut capacity, hedge.clone());
    }
    capacity
}

fn active_commitments(query: &Query<'_>) -> HashSet<SoftRequestCommitment> {
    query
        .inputs
        .in_flight
        .iter()
        .map(|item| {
            SoftRequestCommitment::new(
                item.post().clone(),
                item.identity().source().as_str().to_owned(),
                item.request(),
            )
        })
        .collect()
}

fn ordinary(query: &Query<'_>) -> Capacity {
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
        .map(commitment)
        .collect();
    Capacity {
        requests: posts.len(),
        guarded_requests: 0,
        commitments,
    }
}

fn extend_emergency_current(
    query: &Query<'_>,
    active: &HashSet<SoftRequestCommitment>,
    capacity: &mut Capacity,
) {
    if query.base.mode != ControlMode::Emergency {
        return;
    }
    query
        .base
        .allocations
        .iter()
        .filter(|item| item.post == query.snapshot.playback.current)
        .map(commitment)
        .filter(|item| !active.contains(item))
        .for_each(|item| add_guarded(capacity, item));
}

fn add(capacity: &mut Capacity, commitment: SoftRequestCommitment) {
    if !capacity.commitments.contains(&commitment) {
        capacity.commitments.push(commitment);
        capacity.requests = capacity.requests.saturating_add(1);
    }
}

fn add_guarded(capacity: &mut Capacity, commitment: SoftRequestCommitment) {
    if !capacity.commitments.contains(&commitment) {
        capacity.commitments.push(commitment);
        capacity.guarded_requests = capacity.guarded_requests.saturating_add(1);
    }
}

fn commitment(item: &Allocation) -> SoftRequestCommitment {
    SoftRequestCommitment::new(item.post.clone(), item.source.clone(), item.request)
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
