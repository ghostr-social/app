use super::current_origin_commitment_fixture::contended_state;
use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::{ActionId, ByteRange};

#[test]
fn fully_cached_current_preserves_useful_outgoing_work_on_the_same_origin() {
    let mut input = contended_state();
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 1);
    assert_eq!(plan.retained[0].action_id, ActionId::new(7));
    assert!(plan
        .allocations
        .iter()
        .all(|work| work.post != input.playback.current));
}
