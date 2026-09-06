use super::warp_current_owned_fixture::{history_admitted, state};
use crate::adaptive::AdaptivePlayabilityPolicy;

#[test]
fn current_owning_its_origin_does_not_reserve_an_impossible_second_request() {
    let state = state();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    assert!(
        base.allocations.iter().any(|work| {
            work.post == state.playback.current && work.request.requested_bytes().start >= 65_536
        }),
        "fixture must require current bytes beyond its active response"
    );
    assert!(
        history_admitted(&state, 1),
        "independent history preparation must use the spare request"
    );
}
