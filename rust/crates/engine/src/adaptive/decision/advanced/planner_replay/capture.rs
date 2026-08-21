use super::{
    RecordedNetworkState, RecordedPlannerConfig, RecordedPlannerReplayCapsule, MODEL_BYTE_LIMIT,
};
use crate::adaptive::decision::privacy::DecisionPrivacy;
use crate::adaptive::{RecordedResourcePrices, WarpPlanningDecision};

const SOURCE_LIMIT: usize = 64;
pub(in crate::adaptive::decision) fn capture(
    decision: &WarpPlanningDecision,
    privacy: &DecisionPrivacy,
) -> Option<RecordedPlannerReplayCapsule> {
    let value = decision.planner_replay.as_ref()?;
    let projected = projected(value, privacy)?;
    if !recordable(value, &projected) {
        return Some(RecordedPlannerReplayCapsule::incomplete());
    }
    Some(projected)
}

fn projected(
    value: &crate::adaptive::PlannerReplayCapsule,
    privacy: &DecisionPrivacy,
) -> Option<RecordedPlannerReplayCapsule> {
    let post = |item: &str| privacy.post(item);
    let source = |item: &str| privacy.source(item);
    let aliases: Vec<_> = value
        .sources()
        .iter()
        .map(|item| (item.clone(), source(item)))
        .collect();
    let base = value.base().replay_project(&post, &source);
    let context = value.context().replay_project(&post, &source);
    let origins = value
        .origins()
        .replay_project(&aliases, &|item| privacy.model_key(item));
    Some(RecordedPlannerReplayCapsule {
        complete: value.complete(),
        base: Some(base),
        context: Some(context),
        origins: Some(origins),
        config: Some(config(value.config())?),
        controller_prices: Some(prices(value.controller_prices())),
        network: Some(network(value.network())),
        price_epoch: Some(value.price_epoch()),
    })
}

fn recordable(
    value: &crate::adaptive::PlannerReplayCapsule,
    projected: &RecordedPlannerReplayCapsule,
) -> bool {
    value.complete()
        && value.sources().len() <= SOURCE_LIMIT
        && value.base().replay_bounded()
        && value.context().replay_bounded()
        && projected.origins.as_ref().is_some_and(|origins| {
            origins.replay_bounded()
                && serde_json::to_vec(origins)
                    .is_ok_and(|encoded| encoded.len() <= MODEL_BYTE_LIMIT)
        })
}

fn config(value: crate::adaptive::WarpPlannerConfig) -> Option<RecordedPlannerConfig> {
    let (depth, width, expansions, latency) = value.beam.replay_parts();
    Some(RecordedPlannerConfig {
        beam_depth: u64::try_from(depth).ok()?,
        beam_width: u64::try_from(width).ok()?,
        beam_expansions: u64::try_from(expansions).ok()?,
        beam_latency_us: latency,
        twin_particles: value.twin.particles,
        twin_tail_bps: value.twin.tail_bps,
        semantic_top_k: u64::try_from(value.semantic_top_k).ok()?,
        semantic_epsilon_micros: value.semantic_epsilon_micros,
        safety_rescue_bps: value.safety_rescue_bps,
        emergency_rescue_bps: value.emergency_rescue_bps,
    })
}

fn prices(value: crate::adaptive::ResourcePrices) -> RecordedResourcePrices {
    RecordedResourcePrices {
        network_micros: value.network_micros,
        storage_micros: value.storage_micros,
        cpu_micros: value.cpu_micros,
        request_micros: value.request_micros,
    }
}

fn network(value: Option<&crate::adaptive::NetworkTokenBucket>) -> RecordedNetworkState {
    let Some(value) = value else {
        return RecordedNetworkState::Uninitialized;
    };
    let (capacity, refill_per_second, tokens, updated_at_ms) = value.replay_parts();
    RecordedNetworkState::Initialized {
        capacity,
        refill_per_second,
        tokens,
        updated_at_ms,
    }
}

impl RecordedPlannerReplayCapsule {
    fn incomplete() -> Self {
        Self {
            complete: false,
            base: None,
            context: None,
            origins: None,
            config: None,
            controller_prices: None,
            network: None,
            price_epoch: None,
        }
    }
}
