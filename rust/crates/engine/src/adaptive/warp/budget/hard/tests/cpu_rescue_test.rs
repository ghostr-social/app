use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode, ActionValue, TransformKind};
use crate::PostId;

#[test]
fn ordinary_local_work_cannot_auction_reserved_cpu() {
    let rescue = transform(1);
    let mut budget = HardBudget::new(ResourceCost::new(0, 0, 1, 0), 0)
        .protect(std::slice::from_ref(&rescue))
        .unwrap();

    assert_eq!(
        budget.consume_action(&transform(2)),
        Err(BudgetDenial::RescueReserve)
    );
}

fn transform(id: u16) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(format!("p{id}")),
        ActionKind::Transform(TransformKind::Remux),
        ActionValue::default(),
    )
    .with_resources(ResourceCost::new(0, 0, 1, 0))
}
