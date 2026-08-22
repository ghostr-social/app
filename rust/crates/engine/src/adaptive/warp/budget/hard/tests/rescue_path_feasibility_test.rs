use super::request;
use crate::adaptive::warp::budget::hard::{HardBudget, ResourceCost};

#[test]
fn individually_feasible_steps_cannot_overbook_the_rescue_path() {
    let path = [request(1, 60), request(2, 60)];
    let budget = HardBudget::new(ResourceCost::new(2, 100, 0, 2), 2);

    assert!(budget.protect(&path).is_none());
}
