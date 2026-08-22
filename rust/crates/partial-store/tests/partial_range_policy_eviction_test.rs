mod store_fixture;

use std::sync::Arc;
use store_fixture::{discard, plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn policy_eviction_removes_only_the_selected_sparse_range() {
    let root = temp_root("ghostr-policy-range-eviction");
    let used = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), used.clone());
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("seed range");
    store.set_total_len("clip", 12).await.expect("total");

    let eviction = 4..8;
    let freed = store
        .evict_ranges("clip", std::slice::from_ref(&eviction))
        .await
        .expect("evict range");

    assert_eq!(freed.freed_bytes(), 4);
    assert_eq!(freed.ranges().len(), 1);
    assert_eq!(freed.ranges()[0], 4..8);
    assert_eq!(*used.lock().await, 8);
    assert_eq!(
        store.present_ranges("clip").await.unwrap(),
        vec![0..4, 8..12]
    );
    assert_eq!(
        store.read_range("clip", 0..4).await.unwrap().unwrap(),
        b"abcd"
    );
    assert_eq!(store.read_range("clip", 4..8).await.unwrap(), None);
    assert_eq!(
        store.read_range("clip", 8..12).await.unwrap().unwrap(),
        b"ijkl"
    );
    discard(&root);
}
