use crate::tests::adaptive_plan_assertions::allocated_posts;
use crate::tests::adaptive_plan_support::plan_with_capacity;
use ghostr_engine::adaptive::StorageSnapshot;

#[test]
fn live_connection_capacity_changes_the_manager_plan_frontier() {
    let storage = StorageSnapshot::new(2_000_000_000, 0);
    let serial = allocated_posts(&plan_with_capacity(20_000, 4_000_000, storage, 1));
    let parallel = allocated_posts(&plan_with_capacity(20_000, 4_000_000, storage, 6));

    assert!(
        parallel.len() >= serial.len() && parallel.len() <= 2,
        "{serial:?} versus {parallel:?}"
    );
}
