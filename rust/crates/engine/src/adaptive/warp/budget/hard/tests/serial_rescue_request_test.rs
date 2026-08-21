use crate::adaptive::warp::budget::hard::{HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode, ActionValue};
use crate::{ByteRange, PostId};

#[test]
fn serial_rescue_steps_reuse_the_same_authority_slot() {
    let first = request(1, ByteRange::new(0, 1), &[]);
    let second = request(2, ByteRange::new(1, 2), &[1]);
    let mut budget = HardBudget::new(ResourceCost::new(2, 2, 0, 1), 1)
        .protect(&[first.clone(), second.clone()])
        .expect("serial rescue fits one slot");

    assert!(budget.consume_action(&first).is_ok());
    assert!(budget.consume_action(&second).is_ok());
}

fn request(id: u16, bytes: ByteRange, requires: &[u16]) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("p0"),
        ActionKind::FetchRange(bytes),
        ActionValue::default(),
    )
    .with_resources(ResourceCost::new(1, 1, 0, 1))
    .with_origin("https://origin.example/media")
    .requiring(requires)
}
