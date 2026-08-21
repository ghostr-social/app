use super::{node, select_path};
use crate::adaptive::ResourceCost;

#[test]
fn shared_dependency_is_reserved_once_in_topological_order() {
    let path = [
        node(1, ResourceCost::new(30, 30, 0, 1), 0, &[]),
        node(2, ResourceCost::new(10, 10, 0, 1), 0, &[1]),
        node(3, ResourceCost::new(10, 10, 0, 1), 0, &[1]),
        node(4, ResourceCost::new(0, 10, 1, 0), 1_000, &[2, 3]),
    ];
    let limits = ResourceCost::new(50, 60, 1, 1);
    let selected = select_path(&path, limits).unwrap();

    assert_eq!(
        selected
            .steps
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(selected.cost, limits);
}
