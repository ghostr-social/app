use super::current_origin_commitment_fixture::contended_state;
use crate::adaptive::AdaptivePlayabilityPolicy;

#[test]
fn current_startup_displaces_future_work_holding_its_only_origin_slot() {
    let input = contended_state();

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .allocations
        .iter()
        .any(|work| work.post == input.playback.current));
    assert!(
        plan.retained.is_empty(),
        "current needs the occupied origin slot: {plan:#?}"
    );
}
