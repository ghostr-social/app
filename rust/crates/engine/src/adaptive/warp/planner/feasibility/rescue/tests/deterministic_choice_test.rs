use super::{node, select_path};
use crate::adaptive::ResourceCost;

#[test]
fn minimum_rescue_is_stable_when_frontier_order_changes() {
    let cheap = node(1, ResourceCost::new(0, 10, 1, 0), 1_000, &[]);
    let costly = node(2, ResourceCost::new(0, 20, 1, 0), 1_000, &[]);
    let limits = ResourceCost::new(0, 30, 2, 0);
    let first = select_path(&[costly.clone(), cheap.clone()], limits).unwrap();
    let second = select_path(&[cheap, costly], limits).unwrap();

    assert_eq!(first.steps.last().unwrap().id, 1);
    assert_eq!(second.steps.last().unwrap().id, 1);
}
