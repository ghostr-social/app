use super::{active, partial_canonical_focus, provisional_state, NEXT, THIRD};
use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::DataUsageLevel;

pub(in crate::tests) const OBSERVED_AT_MS: u64 = 1_000;

pub(in crate::tests) struct DetachedFuture {
    pub(in crate::tests) state: DeliveryState,
    pub(in crate::tests) active: ActiveAction,
}

pub(in crate::tests) fn detached_next(
    committed_until_ms: u64,
    digest: Option<&str>,
) -> DetachedFuture {
    let mut state = provisional_state(DataUsageLevel::Balanced, None, digest);
    let active = active(&state, NEXT, 1, committed_until_ms);
    assert!(state.apply_focus(partial_canonical_focus(None), OBSERVED_AT_MS));
    DetachedFuture { state, active }
}

pub(in crate::tests) fn handoff_with_expired_third() -> (DeliveryState, [ActiveAction; 2]) {
    let mut state = provisional_state(DataUsageLevel::Balanced, None, None);
    let active = [
        active(&state, THIRD, 1, OBSERVED_AT_MS),
        active(&state, NEXT, 2, 4_000),
    ];
    assert!(state.apply_focus(partial_canonical_focus(None), OBSERVED_AT_MS));
    (state, active)
}
