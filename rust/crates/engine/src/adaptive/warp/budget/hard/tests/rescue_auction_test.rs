use super::request;
use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};

#[test]
fn ordinary_work_cannot_auction_the_pending_rescue_slot() {
    let rescue = request(3, 1);
    let mut budget = HardBudget::new(ResourceCost::new(3, 3, 0, 2), 2)
        .protect(core::slice::from_ref(&rescue))
        .expect("rescue fits");

    assert!(budget.consume_action(&request(1, 1)).is_ok());
    assert_eq!(
        budget.consume_action(&request(2, 1)),
        Err(BudgetDenial::RescueReserve)
    );
    assert!(budget.consume_action(&rescue).is_ok());
}
