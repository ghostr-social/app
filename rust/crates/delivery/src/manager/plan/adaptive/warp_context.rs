use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    AllocationPlan, HeadProbeHistory, PlannerContext, PreviewAvailability, RequestOccupancy,
    TwinEpochs,
};

mod active;
mod cooling;
mod hedge;
mod limits;
#[cfg(test)]
#[path = "warp_context/limits_test.rs"]
mod limits_test;
mod quality;

mod request_capacity;
mod whole_body;

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
    let limits = limits::resolve(state, snapshot, &request_capacity);
    let context = context.with_limits(limits).with_soft_request_capacity(
        request_capacity.ordinary_tokens,
        request_capacity.hls_tokens,
        request_capacity.soft,
    );
    let context = apply_feedback(context, inputs.resource_feedback)
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
    let context = whole_body::apply(context, state, candidate, inputs);
    let context = context
        .with_capability(
            candidate.post.clone(),
            state.planner_capability(&candidate.post, inputs.observed_at_ms),
        )
        .with_head_probe_history(
            candidate.post.clone(),
            head_probe_history(state, candidate, inputs),
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
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
    inputs: &PlanInputs<'_>,
) -> HeadProbeHistory {
    let post = &candidate.post;
    if completed_head_is_current(state, candidate, inputs)
        && !crate::probe::pool::evidence_needs_head_refresh(
            &candidate.evidence,
            historical_size(state, candidate),
        )
    {
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

fn historical_size(
    state: &DeliveryState,
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
) -> bool {
    candidate.preferred_source.as_deref().is_some_and(|source| {
        state
            .catalog()
            .lookup(&candidate.post)
            .and_then(|entry| entry.planning_total_for(source))
            .is_some()
    })
}

fn completed_head_is_current(
    state: &DeliveryState,
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
    inputs: &PlanInputs<'_>,
) -> bool {
    let Some(source) = candidate.preferred_source.as_deref() else {
        return false;
    };
    state
        .catalog()
        .transfer_identity(&candidate.post, source)
        .as_ref()
        .is_some_and(|identity| inputs.completed_head_probes.contains(identity))
}

fn apply_feedback(
    context: PlannerContext,
    feedback: Option<ghostr_engine::adaptive::ResourceFeedback>,
) -> PlannerContext {
    match feedback {
        Some(feedback) => context.with_feedback(feedback),
        None => context,
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
