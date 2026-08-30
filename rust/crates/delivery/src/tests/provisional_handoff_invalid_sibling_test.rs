use crate::tests::provisional_handoff_fixture::handoff_with_expired_third;
use crate::tests::provisional_handoff_plan_fixture::{generated_cancels, plan};
use ghostr_engine::ActionId;
use std::collections::HashSet;

#[test]
fn expiring_far_handoff_does_not_evict_the_valid_nearest_future() {
    let (state, active) = handoff_with_expired_third();
    let work = plan(state, &active, 2);

    assert_eq!(generated_cancels(&work), HashSet::from([ActionId::new(1)]));
    assert!(!work.retained.contains(&ActionId::new(1)));
    assert!(work.retained.contains(&ActionId::new(2)));
}
