use super::current_origin_commitment_fixture::contended_state;
use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::ActionId;

#[test]
fn current_startup_preserves_outgoing_work_on_an_independent_origin() {
    let mut input = contended_state();
    input.candidates[0].origins[0].source = "https://another.example/current.mp4".into();

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(plan
        .allocations
        .iter()
        .any(|work| work.post == input.playback.current));
    assert_eq!(plan.retained.len(), 1);
    assert_eq!(plan.retained[0].action_id, ActionId::new(7));
}
