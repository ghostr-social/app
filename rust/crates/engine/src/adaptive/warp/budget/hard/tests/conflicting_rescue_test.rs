use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode, ActionValue};
use crate::{ByteRange, PostId};

#[test]
fn conflicting_ordinary_work_cannot_invalidate_the_rescue() {
    let rescue = node(1, ActionKind::FetchWhole { maximum_bytes: 1 });
    let ordinary = node(2, ActionKind::FetchRange(ByteRange::new(0, 1)));
    let mut budget = HardBudget::new(ResourceCost::new(2, 2, 0, 2), 2)
        .protect(std::slice::from_ref(&rescue))
        .unwrap();

    assert_eq!(
        budget.consume_action(&ordinary),
        Err(BudgetDenial::RescueReserve)
    );
}

fn node(id: u16, kind: ActionKind) -> ActionNode {
    ActionNode::new(id, PostId::new("post"), kind, ActionValue::default())
        .with_resources(ResourceCost::new(1, 1, 0, 1))
        .with_origin("https://origin.example/media")
}
