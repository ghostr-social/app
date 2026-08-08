mod store_fixture;

use store_fixture::{discard, limits, spaced_store};

/// The reserve is a floor, not a target: the store may spend every byte
/// above it and not one byte of it, however large the budget is.
#[tokio::test]
async fn partial_range_capacity_never_consumes_the_free_space_reserve() {
    let fixture = spaced_store("ghostr-cap-reserve", limits(u64::MAX, 1_000), 1_000);
    let store = &fixture.store;

    let refused = store
        .write_range("clip", 0, &[9; 1])
        .await
        .expect_err("the reserve is not spendable");

    assert!(
        refused.to_string().contains("space"),
        "unhelpful: {refused}"
    );
    assert_eq!(*fixture.used_bytes.lock().await, 0);
    assert!(
        !fixture.root.exists(),
        "a refused write must not create a file"
    );

    fixture.space.set(1_002);
    store
        .write_range("clip", 0, &[9; 2])
        .await
        .expect("the two bytes above the reserve");
    fixture.space.set(1_000); // those two bytes are now spent
    assert_eq!(*fixture.used_bytes.lock().await, 2);

    let exhausted = store
        .write_range("clip", 2, &[9; 1])
        .await
        .expect_err("the reserve itself is never spendable");

    assert!(
        exhausted.to_string().contains("space"),
        "unhelpful: {exhausted}"
    );
    assert_eq!(*fixture.used_bytes.lock().await, 2);
    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        vec![0..2]
    );

    discard(&fixture.root);
}
