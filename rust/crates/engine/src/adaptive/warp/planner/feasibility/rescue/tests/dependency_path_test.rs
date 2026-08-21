use super::{exact_limits, node, select_path};
use crate::adaptive::ResourceCost;

#[test]
fn rescue_reserves_the_transitive_dependency_closure() {
    let path = [
        node(1, ResourceCost::new(40, 40, 0, 1), 0, &[]),
        node(2, ResourceCost::new(20, 20, 0, 1), 0, &[1]),
        node(3, ResourceCost::new(0, 30, 5, 0), 1_000, &[2]),
    ];

    let selected = select_path(&path, exact_limits()).expect("rescue path");
    assert_eq!(
        selected
            .steps
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(selected.cost, exact_limits());
}
