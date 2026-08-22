use super::request;
use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};

#[test]
fn a_rescue_step_is_charged_exactly_once() {
    let rescue = request(1, 1);
    let mut budget = HardBudget::new(ResourceCost::new(1, 1, 0, 1), 1)
        .protect(std::slice::from_ref(&rescue))
        .unwrap();

    assert!(budget.consume_action(&rescue).is_ok());
    assert_eq!(
        budget.consume_action(&request(2, 1)),
        Err(BudgetDenial::HardLimit)
    );
}
