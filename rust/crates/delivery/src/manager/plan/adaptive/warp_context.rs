use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AllocationPlan, HeadProbeHistory, PlannerContext, PlannerLimits, RequestOccupancy,
    ResourceFeedback, ResourceObservation, TwinEpochs,
};

mod active;
mod cooling;
mod hedge;

mod request_capacity;

pub(super) fn build(
    state: &DeliveryState,
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
    base: &AllocationPlan,
    inputs: &PlanInputs<'_>,
) -> (
    PlannerContext,
    RequestOccupancy,
    Vec<crate::manager::hedge_tail::HedgeTailWake>,
) {
    let occupancy = request_occupancy(inputs);
    let context = snapshot.candidates.iter().fold(
        PlannerContext::explicitly_unavailable(snapshot),
        |context, candidate| {
            context
                .with_capability(
                    candidate.post.clone(),
                    state.planner_capability(&candidate.post, inputs.observed_at_ms),
                )
                .with_head_probe_history(
                    candidate.post.clone(),
                    head_probe_history(state, &candidate.post, inputs),
                )
        },
    );
    let context = cooling::apply(context, snapshot, inputs);
    let active = active::BuildInput {
        state,
        snapshot,
        base,
        inputs,
    };
    let (context, tails, hedge_soft) = active::apply(context, active);
    let request_capacity = request_capacity::resolve(request_capacity::Query {
        state,
        snapshot,
        base,
        inputs,
        occupancy: &occupancy,
        hedge_soft: &hedge_soft,
    });
    let limits = limits(state, snapshot, request_capacity.tokens);
    let context = context
        .with_limits(limits)
        .with_soft_request_capacity(request_capacity.ordinary_tokens, request_capacity.soft)
        .with_feedback(feedback(
            snapshot,
            &occupancy,
            inputs.measured_network_bytes_per_second,
            limits.cpu_ms,
        ))
        .with_request_occupancy(occupancy.clone())
        .with_epochs(epochs(state, inputs));
    (context, occupancy, tails)
}

fn head_probe_history(
    state: &DeliveryState,
    post: &ghostr_engine::PostId,
    inputs: &PlanInputs<'_>,
) -> HeadProbeHistory {
    if inputs.completed_head_probes.contains(post) {
        return HeadProbeHistory::Completed;
    }
    let active = inputs.active_head_probes.iter().any(|identity| {
        identity.post() == post
            && state
                .catalog()
                .transfer_identity(post, identity.source().as_str())
                .as_ref()
                == Some(identity)
    });
    match active {
        true => HeadProbeHistory::Active,
        false => HeadProbeHistory::Unobserved,
    }
}

fn limits(
    state: &DeliveryState,
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
    request_tokens: u16,
) -> PlannerLimits {
    let rate = snapshot.network.throughput_bps.saturating_div(8).max(1);
    PlannerLimits {
        network_burst_bytes: rate.saturating_mul(2).max(snapshot.request_slice_bytes),
        network_rate_bytes_per_second: rate,
        cpu_ms: state
            .transform_profile()
            .map_or(0, |profile| profile.limits().cpu_ms()),
        request_tokens,
        per_origin_requests: snapshot
            .network
            .per_authority_request_limit
            .min(u16::MAX as usize) as u16,
    }
}

fn feedback(
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
    occupancy: &RequestOccupancy,
    measured_network_bytes_per_second: u64,
    cpu_target_ms: u64,
) -> ResourceFeedback {
    let rate = snapshot.network.throughput_bps.saturating_div(8);
    ResourceFeedback {
        actual: ResourceObservation::new(
            measured_network_bytes_per_second,
            snapshot.storage.used_bytes,
            0,
            occupancy.total() as u64,
        ),
        target: ResourceObservation::new(
            rate,
            snapshot.storage.budget_bytes.saturating_mul(9) / 10,
            cpu_target_ms,
            snapshot.network.connection_capacity.max(1) as u64,
        ),
    }
}

fn request_occupancy(inputs: &PlanInputs<'_>) -> RequestOccupancy {
    let bodies = inputs
        .in_flight
        .iter()
        .map(|active| active.identity().source().as_str());
    let probes = inputs
        .active_head_probes
        .iter()
        .map(|identity| identity.source().as_str());
    RequestOccupancy::from_sources(bodies.chain(probes))
}

fn epochs(state: &DeliveryState, inputs: &PlanInputs<'_>) -> TwinEpochs {
    TwinEpochs::new(
        state
            .catalog()
            .reliability_revision()
            .saturating_add(state.client_capability_revision()),
        0,
        inputs.capacity_revision,
    )
}
