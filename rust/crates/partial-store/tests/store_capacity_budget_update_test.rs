mod store_space;

use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use store_space::{limits, temp_root, FakeSpace};

#[tokio::test]
async fn store_capacity_budget_change_is_immediate_and_advances_its_generation() {
    let root = temp_root("ghostr-cap-budget-generation");
    let capacity = StoreCapacity::new(limits(800, 0), FakeSpace::new(10_000));
    assert_eq!(capacity.cap(&root, 0).await, 800);
    let measured = capacity.generation();

    let changed = capacity.set_budget(400).expect("positive budget");

    assert!(changed);
    assert!(capacity.generation() > measured);
    assert_eq!(capacity.cap(&root, 0).await, 400);
    let updated = capacity.generation();
    assert!(!capacity.set_budget(400).expect("same positive budget"));
    assert_eq!(
        capacity.generation(),
        updated,
        "a no-op is not a new decision"
    );
}
