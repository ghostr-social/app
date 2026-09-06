use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, PlannerContext, PlannerLimits, PlayabilitySnapshot,
    PlayerPreparation, WarpPlanner, WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision,
    REQUEST_SLICE_BYTES,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

pub(super) fn state() -> PlayabilitySnapshot {
    let mut state = snapshot(2, 20_000_000, 20_000, 0);
    state.navigation.backward_swipes_per_minute = 30;
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    state.candidates[0].finalized = true;
    let previous = &mut state.candidates[1];
    previous.feed_offset = FeedOffset::new(-1);
    previous.view_probability = state.navigation.view_probability(previous.feed_offset);
    previous.player_preparation = PlayerPreparation::Unverified;
    state
}

pub(super) fn plan(state: &PlayabilitySnapshot, config: WarpPlannerConfig) -> WarpPlanningDecision {
    let base = AdaptivePlayabilityPolicy.plan(state);
    WarpPlanner::new(config).plan(WarpPlannerInput::new(
        state,
        &base,
        &OriginModel::default(),
        &context(state),
    ))
}

pub(super) fn context(state: &PlayabilitySnapshot) -> PlannerContext {
    PlannerContext::explicitly_unavailable(state).with_limits(PlannerLimits {
        network_burst_bytes: REQUEST_SLICE_BYTES,
        network_rate_bytes_per_second: REQUEST_SLICE_BYTES,
        cpu_ms: 0,
        request_tokens: 1,
        per_origin_requests: 1,
    })
}

pub(super) fn starting_state() -> PlayabilitySnapshot {
    let mut state = state();
    state.playback.phase = crate::playback::PlaybackPhase::Starting;
    state.playback.buffer_ahead_ms = 0;
    let current = &mut state.candidates[0];
    current.present.clear();
    current.finalized = false;
    current.layout = crate::adaptive::MediaLayout::Unknown;
    current.player_preparation = PlayerPreparation::Unverified;
    state
}
