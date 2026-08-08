mod store_fixture;

use store_fixture::{discard, limits, spaced_store};

/// Other apps eat the same filesystem, so the cap can fall under what
/// the store already holds. Refusing new writes is not enough then: the
/// store gives bytes back, least recently used first.
#[tokio::test]
async fn partial_range_capacity_evicts_least_recently_used_when_free_space_shrinks() {
    let fixture = spaced_store("ghostr-cap-evict", limits(1_000_000, 1_000), 3_000);
    let store = &fixture.store;
    store
        .write_range("older", 0, &[1; 400])
        .await
        .expect("older");
    store
        .write_range("newer", 0, &[2; 400])
        .await
        .expect("newer");
    assert_eq!(*fixture.used_bytes.lock().await, 800);

    fixture.space.set(600);
    let evicted = store.enforce_capacity().await;

    assert_eq!(evicted, 400, "exactly the shortfall");
    assert_eq!(*fixture.used_bytes.lock().await, 400);
    assert_eq!(
        store.present_ranges("older").await.expect("older ranges"),
        Vec::new(),
        "the least recently used video is gone"
    );
    assert!(!fixture.root.join("older.part").exists(), "bytes on disk");
    assert!(!fixture.root.join("older.ranges.json").exists(), "manifest");
    assert_eq!(
        store.present_ranges("newer").await.expect("newer ranges"),
        vec![0..400],
        "the most recently used video survives"
    );
    assert_eq!(
        store.read_range("newer", 0..400).await.expect("read newer"),
        Some(vec![2; 400])
    );

    discard(&fixture.root);
}
