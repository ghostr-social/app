use super::types::WarpPlannerInput;
use crate::adaptive::{TwinEpochs, TwinState};

pub(super) fn state(input: &WarpPlannerInput<'_>) -> TwinState {
    TwinState::new(
        input.snapshot.playback.buffer_ahead_ms,
        input.snapshot.network.throughput_bps,
        input.snapshot.network.rtt_ms,
        input.context.remaining_request_slots(),
    )
    .with_ready_coverage(input.base.ready_reserve.ready_coverage_ms)
    .with_cache_bytes(input.snapshot.storage.used_bytes)
    .with_swipe_rate(input.snapshot.navigation.forward_swipes_per_minute)
}

pub(super) fn epochs(input: &WarpPlannerInput<'_>, price_epoch: u64) -> TwinEpochs {
    TwinEpochs::new(
        input.context.epochs.evidence,
        input.context.epochs.model,
        input.context.epochs.budget.saturating_add(price_epoch),
    )
}
