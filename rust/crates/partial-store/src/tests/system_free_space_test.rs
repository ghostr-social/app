use crate::partial_range_store::free_space::{FreeSpace as _, SystemFreeSpace};
use std::path::Path;

#[test]
fn system_free_space_measures_the_nearest_existing_ancestor() {
    let absent_descendant = Path::new("target/absent/free-space-probe");

    assert!(
        SystemFreeSpace.available_bytes(absent_descendant).is_some(),
        "the system adapter should measure an existing ancestor"
    );
}
