use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    ActivePlannerContext, AllocationPlan, HeadProbeHistory, PlannerContext, PlannerLimits,
    ResourceFeedback, ResourceObservation, TwinEpochs,
};

pub(super) fn build(
    state: &DeliveryState,
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
    base: &AllocationPlan,
    inputs: &PlanInputs<'_>,
) -> PlannerContext {
    let context = PlannerContext::explicitly_unavailable(snapshot)
        .with_limits(limits(snapshot))
        .with_feedback(feedback(snapshot))
        .with_epochs(epochs(state, snapshot));
    let context = snapshot
        .candidates
        .iter()
        .fold(context, |context, candidate| {
            let context = context.with_capability(
                candidate.post.clone(),
                state.planner_capability(&candidate.post, inputs.observed_at_ms),
            );
            match inputs.completed_head_probes.contains(&candidate.post) {
                true => context
                    .with_head_probe_history(candidate.post.clone(), HeadProbeHistory::Completed),
                false => context,
            }
        });
    inputs.in_flight.iter().fold(context, |context, active| {
        let advantage = continuation_advantage(base, active.action_id());
        context.with_active(
            ActivePlannerContext::new(active.action_id()).with_continuation_advantage(advantage),
        )
    })
}

fn limits(snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot) -> PlannerLimits {
    let rate = snapshot.network.throughput_bps.saturating_div(8).max(1);
    PlannerLimits {
        network_burst_bytes: rate.saturating_mul(2).max(snapshot.request_slice_bytes),
        network_rate_bytes_per_second: rate,
        cpu_ms: 0,
        request_tokens: snapshot.network.connection_capacity.min(u16::MAX as usize) as u16,
        per_origin_requests: snapshot.network.connection_ceiling.min(u16::MAX as usize) as u16,
    }
}

fn feedback(snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot) -> ResourceFeedback {
    let active = snapshot
        .candidates
        .iter()
        .map(|candidate| candidate.in_flight.len() as u64)
        .sum::<u64>();
    let rate = snapshot.network.throughput_bps.saturating_div(8);
    ResourceFeedback {
        actual: ResourceObservation::new(
            if active == 0 { 0 } else { rate },
            snapshot.storage.used_bytes,
            0,
            active,
        ),
        target: ResourceObservation::new(
            rate,
            snapshot.storage.budget_bytes.saturating_mul(9) / 10,
            1,
            snapshot.network.connection_capacity.max(1) as u64,
        ),
    }
}

fn epochs(
    state: &DeliveryState,
    snapshot: &ghostr_engine::adaptive::PlayabilitySnapshot,
) -> TwinEpochs {
    TwinEpochs::new(
        state
            .catalog()
            .reliability_revision()
            .saturating_add(state.client_capability_revision()),
        0,
        snapshot
            .storage
            .used_bytes
            .saturating_add(snapshot.storage.budget_bytes),
    )
}

fn continuation_advantage(base: &AllocationPlan, action: ghostr_engine::ActionId) -> i64 {
    match base.retained.iter().any(|item| item.action_id == action) {
        true => 100_000,
        false => -100_000,
    }
}
