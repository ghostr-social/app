use super::{exact_limits, node, select_path};
use crate::adaptive::ResourceCost;
use crate::PostId;

#[test]
fn every_rescue_dependency_requires_semantic_admission() {
    let mut excluded = node(1, ResourceCost::new(20, 20, 0, 1), 0, &[]);
    excluded.post = PostId::new("excluded");
    let terminal = node(2, ResourceCost::new(20, 20, 0, 1), 1_000, &[1]);

    assert!(select_path(&[excluded, terminal], exact_limits()).is_none());
}
