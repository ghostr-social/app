use super::{exact_limits, node, select_path};
use crate::adaptive::ResourceCost;

#[test]
fn missing_or_cyclic_dependencies_cannot_form_a_rescue() {
    let missing = node(1, ResourceCost::new(1, 1, 0, 0), 1_000, &[9]);
    assert!(select_path(&[missing], exact_limits()).is_none());

    let first = node(1, ResourceCost::new(1, 1, 0, 0), 0, &[2]);
    let second = node(2, ResourceCost::new(1, 1, 0, 0), 1_000, &[1]);
    assert!(select_path(&[first, second], exact_limits()).is_none());
}
