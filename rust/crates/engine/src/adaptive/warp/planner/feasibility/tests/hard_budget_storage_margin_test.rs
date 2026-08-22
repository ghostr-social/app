use super::{feasible, node};
use crate::adaptive::ResourceCost;

#[test]
fn sub_percent_storage_budget_does_not_reserve_a_whole_byte() {
    let limits = ResourceCost::new(0, 16, 0, 0);
    let action = node(1, limits, 10, &[]);

    let result = feasible(&[action], limits);

    assert_eq!(result.nodes.len(), 1);
}
