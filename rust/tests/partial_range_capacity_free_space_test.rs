mod store_space;

use store_space::{discard, limits, spaced_store};

/// The configured budget is only half the answer: what the device can
/// actually spare decides the cap, and it is re-measured rather than
/// read once at startup.
#[tokio::test]
async fn partial_range_effective_capacity_follows_free_space() {
    let fixture = spaced_store("ghostr-cap-free-space", limits(10_000, 1_000), 4_000);
    let store = &fixture.store;

    assert_eq!(store.effective_capacity().await, 3_000, "free space caps");

    fixture.space.set(20_000);
    assert_eq!(store.effective_capacity().await, 10_000, "budget caps");

    fixture.space.set(900);
    assert_eq!(store.effective_capacity().await, 0, "below the reserve");

    fixture.space.set(4_000);
    store.write_range("clip", 0, &[7; 500]).await.expect("write");
    fixture.space.set(3_500); // the file system really lost those bytes
    assert_eq!(
        store.effective_capacity().await,
        3_000,
        "spending the store's own bytes does not move the cap"
    );

    fixture.space.set(700); // another app took the rest
    assert_eq!(store.effective_capacity().await, 200);
    let refused = store
        .write_range("clip", 500, &[7; 500])
        .await
        .expect_err("write past the cap");
    assert!(refused.to_string().contains("space"), "unhelpful: {refused}");

    fixture.space.set(3_500);
    store
        .write_range("clip", 500, &[7; 500])
        .await
        .expect("write once free space returns");
    assert_eq!(*fixture.used_bytes.lock().await, 1_000);

    discard(&fixture.root);
}
