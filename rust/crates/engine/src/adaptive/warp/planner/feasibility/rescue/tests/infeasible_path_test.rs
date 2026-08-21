use super::{node, select_path};
use crate::adaptive::ResourceCost;

#[test]
fn aggregate_hard_resource_failure_rejects_the_rescue_path() {
    let path = [
        node(1, ResourceCost::new(1, 60, 0, 1), 0, &[]),
        node(2, ResourceCost::new(1, 60, 0, 1), 1_000, &[1]),
    ];
    let limits = ResourceCost::new(2, 100, 0, 2);

    assert!(select_path(&path, limits).is_none());
}
