//! Reaching the cap is not a refusal. A write that does not fit gives
//! back the least recently used video the viewer is not watching and
//! then lands; only the write's own key and leased videos are out of
//! reach, so nothing the user is watching is pulled away.

mod store_space;

use store_space::{discard, limits, spaced_store};

#[tokio::test]
async fn partial_range_write_evicts_unleased_content_instead_of_refusing() {
    let fixture = spaced_store("ghostr-write-evict", limits(1_000, 0), 100_000);
    let store = &fixture.store;
    store.write_range("cold", 0, &[1; 400]).await.expect("cold");
    store
        .write_range("watched", 0, &[2; 400])
        .await
        .expect("watched");
    let lease = store.lease("watched");

    store
        .write_range("hot", 0, &[3; 400])
        .await
        .expect("the full store must make room for the next chunk");

    assert_eq!(store.refusals(), 0, "eviction, not refusal");
    assert_eq!(
        store.present_ranges("cold").await.expect("cold ranges"),
        Vec::new(),
        "the coldest unleased video paid for the write"
    );
    assert_eq!(
        store
            .present_ranges("watched")
            .await
            .expect("watched ranges"),
        vec![0..400],
        "the leased video is never evicted, however old"
    );
    assert_eq!(
        store.present_ranges("hot").await.expect("hot ranges"),
        vec![0..400]
    );
    assert_eq!(*fixture.used_bytes.lock().await, 800);
    assert!(
        store.effective_capacity().await >= 800,
        "the budget still holds"
    );

    drop(lease);
    discard(&fixture.root);
}
