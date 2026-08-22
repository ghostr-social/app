use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AllocationPlan, HeadProbeHistory, PlannerContext, PlannerLimits, PreviewAvailability,
    RequestOccupancy, ResourceFeedback, ResourceObservation, ShadowPriceController, TwinEpochs,
};

mod active;
mod cooling;
mod hedge;
mod quality;

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
        PlannerContext::explicitly_unavailable(snapshot)
            .with_network_class(state.network_class())
            .with_segmented_storage_available_bytes(inputs.segmented_storage_available_bytes),
        |context, candidate| candidate_context(context, state, candidate, inputs),
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
    let limits = limits(state, snapshot, &request_capacity);
    let context = context
        .with_limits(limits)
        .with_soft_request_capacity(
            request_capacity.ordinary_tokens,
            request_capacity.hls_tokens,
            request_capacity.soft,
        )
        .with_feedback(feedback(FeedbackInput {
            snapshot,
            occupancy: &occupancy,
            measured_network_bytes_per_second: inputs.measured_network_bytes_per_second,
            measured_transform_cpu_ms: inputs.measured_transform_cpu_ms,
            cpu_target_ms: limits.cpu_ms,
            request_target: u64::from(request_capacity.tokens),
        }))
        .with_request_occupancy(occupancy.clone())
        .with_epochs(epochs(state, inputs));
    (context, occupancy, tails)
}

fn candidate_context(
    context: PlannerContext,
    state: &DeliveryState,
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
    inputs: &PlanInputs<'_>,
) -> PlannerContext {
    let context = quality::apply(context, state, &candidate.post);
    let context = context
        .with_capability(
            candidate.post.clone(),
            state.planner_capability(&candidate.post, inputs.observed_at_ms),
        )
        .with_head_probe_history(
            candidate.post.clone(),
            head_probe_history(state, &candidate.post, inputs),
        );
    let Some(preview) = state
        .catalog()
        .lookup(&candidate.post)
        .and_then(|entry| entry.preview())
    else {
        return context;
    };
    context.with_preview(
        candidate.post.clone(),
        PreviewAvailability::inline_blurhash(preview),
    )
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
    request_capacity: &request_capacity::RequestCapacity,
) -> PlannerLimits {
    let rate = snapshot.network.throughput_bps.saturating_div(8).max(1);
    let baseline_burst = rate.saturating_mul(2).max(snapshot.request_slice_bytes);
    PlannerLimits {
        network_burst_bytes: baseline_burst.max(request_capacity::hls_burst_floor(
            snapshot,
            request_capacity.hls_tokens,
        )),
        network_rate_bytes_per_second: rate,
        cpu_ms: state
            .transform_profile()
            .map_or(0, |profile| profile.limits().cpu_ms()),
        request_tokens: request_capacity.tokens,
        per_origin_requests: snapshot
            .network
            .per_authority_request_limit
            .min(u16::MAX as usize) as u16,
    }
}

struct FeedbackInput<'a> {
    snapshot: &'a ghostr_engine::adaptive::PlayabilitySnapshot,
    occupancy: &'a RequestOccupancy,
    measured_network_bytes_per_second: u64,
    measured_transform_cpu_ms: Option<u64>,
    cpu_target_ms: u64,
    request_target: u64,
}

fn feedback(input: FeedbackInput<'_>) -> ResourceFeedback {
    let rate = input.snapshot.network.throughput_bps.saturating_div(8);
    let cpu_target = input.measured_transform_cpu_ms.map_or(0, |_| {
        ShadowPriceController::cpu_operating_target_ms(input.cpu_target_ms)
    });
    ResourceFeedback {
        actual: ResourceObservation::new(
            input.measured_network_bytes_per_second,
            input.snapshot.storage.used_bytes,
            input.measured_transform_cpu_ms.unwrap_or_default(),
            input.occupancy.total() as u64,
        ),
        target: ResourceObservation::new(
            rate,
            input.snapshot.storage.budget_bytes.saturating_mul(9) / 10,
            cpu_target,
            input.request_target.max(1),
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
    let hls = inputs.active_hls_sources.iter().map(String::as_str);
    RequestOccupancy::from_sources(bodies.chain(probes).chain(hls))
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
