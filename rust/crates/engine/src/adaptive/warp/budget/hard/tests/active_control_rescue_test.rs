use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode, ActionValue};
use crate::{ActionId, PostId};

#[test]
fn cancelling_an_active_request_cannot_destroy_its_reserved_promotion() {
    let active = ActionId::new(7);
    let promote = node(
        1,
        ActionKind::Promote {
            active,
            maximum_bytes: 10,
        },
    );
    let cancel = node(2, ActionKind::Cancel(active));
    let mut budget = HardBudget::unlimited()
        .protect(core::slice::from_ref(&promote))
        .expect("valid test fixture");

    assert_eq!(
        budget.consume_action(&cancel),
        Err(BudgetDenial::RescueReserve)
    );
}

fn node(id: u16, kind: ActionKind) -> ActionNode {
    ActionNode::new(id, PostId::new("post"), kind, ActionValue::default())
        .with_resources(ResourceCost::default())
}
