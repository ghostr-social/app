use crate::adaptive::warp::budget::hard::{HardBudget, ResourceCost};
use crate::adaptive::{ActionKind, ActionNode, ActionValue, TransformKind};
use crate::PostId;

#[test]
fn local_rescue_needs_no_synthetic_request_slot() {
    let rescue = ActionNode::new(
        1,
        PostId::new("post"),
        ActionKind::Transform(TransformKind::Remux),
        ActionValue::from_net_micros(1),
    )
    .with_resources(ResourceCost::new(0, 16, 4, 0));
    let budget = HardBudget::new(ResourceCost::new(0, 16, 4, 0), 0);
    let mut protected = budget
        .protect(core::slice::from_ref(&rescue))
        .expect("valid test fixture");

    assert!(protected.consume_action(&rescue).is_ok());
}
