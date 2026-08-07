mod store_space;

use store_space::{discard, limits, spaced_store};

#[tokio::test]
async fn partial_range_store_applies_shrinks_now_and_expansions_to_future_writes() {
    let fixture = spaced_store("ghostr-live-budget", limits(800, 0), 10_000);
    let store = &fixture.store;
    store
        .write_range("older", 0, &[1; 400])
        .await
        .expect("older");
    store
        .write_range("newer", 0, &[2; 400])
        .await
        .expect("newer");

    store
        .set_storage_budget(400)
        .await
        .expect("shrink storage budget");

    assert_eq!(*fixture.used_bytes.lock().await, 400);
    assert_eq!(store.present_ranges("older").await.expect("older"), vec![]);
    assert_eq!(
        store.present_ranges("newer").await.expect("newer"),
        vec![0..400]
    );

    store
        .set_storage_budget(800)
        .await
        .expect("expand storage budget");
    store
        .write_range("future", 0, &[3; 400])
        .await
        .expect("future");

    assert_eq!(*fixture.used_bytes.lock().await, 800);
    assert_eq!(
        store.present_ranges("future").await.expect("future"),
        vec![0..400]
    );
    discard(&fixture.root);
}
