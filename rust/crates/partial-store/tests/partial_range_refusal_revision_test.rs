mod store_fixture;

use ghostr_partial_store::partial_range_store::OutOfSpace;
use std::time::Duration;
use store_fixture::{discard, limits, paced_store};

#[tokio::test]
async fn refusal_preserves_a_capacity_change_during_its_decision() {
    let fixture = paced_store("ghostr-refusal-revision", limits(16, 0), 16, Duration::ZERO);
    fixture.store.write_range("hot", 0, &[1; 8]).await.unwrap();
    fixture.space.set(0);
    let changes = fixture.store.capacity_changes();

    let refusal = fixture
        .store
        .write_range("hot", 8, &[2; 8])
        .await
        .unwrap_err()
        .downcast::<OutOfSpace>()
        .unwrap();

    assert!(
        changes.has_changed().unwrap(),
        "measurement changed capacity"
    );
    assert!(
        refusal.capacity_revision().value() < *changes.borrow(),
        "the waiter must still observe a change made during admission"
    );
    discard(&fixture.root);
}
