use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode, ActionValue};
use crate::{ByteRange, PostId};

#[test]
fn ordinary_work_cannot_take_the_rescues_authority_slot() {
    let rescue = request(1, "https://a.example/rescue");
    let same = request(2, "https://a.example/ordinary");
    let other = request(3, "https://b.example/ordinary");
    let mut budget = HardBudget::new(ResourceCost::new(2, 2, 0, 2), 2)
        .protect(std::slice::from_ref(&rescue))
        .unwrap();

    assert_eq!(
        budget.consume_action(&same),
        Err(BudgetDenial::RescueReserve)
    );
    assert!(budget.consume_action(&other).is_ok());
}

fn request(id: u16, origin: &str) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(format!("p{id}")),
        ActionKind::FetchRange(ByteRange::new(0, 1)),
        ActionValue::default(),
    )
    .with_resources(ResourceCost::new(1, 1, 0, 1))
    .with_origin(origin)
}
