use crate::tests::store_fixture::{discard, plain_store, temp_root};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn policy_eviction_removes_only_the_selected_sparse_range() {
    let root = temp_root("ghostr-policy-range-eviction");
    let used = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), std::sync::Arc::clone(&used));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("seed range");
    store.set_total_len("clip", 12).await.expect("total");

    let eviction = 4..8;
    let freed = store
        .evict_ranges("clip", core::slice::from_ref(&eviction))
        .await
        .expect("evict range");

    assert_eq!(freed.freed_bytes(), 4);
    assert_eq!(freed.ranges().len(), 1);
    assert_eq!(freed.ranges()[0], 4..8);
    assert_eq!(*used.lock().await, 8);
    assert_eq!(
        store
            .present_ranges("clip")
            .await
            .expect("valid test fixture"),
        vec![0..4, 8..12]
    );
    assert_eq!(
        store
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture")
            .expect("valid test fixture"),
        b"abcd"
    );
    assert_eq!(
        store
            .read_range("clip", 4..8)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(
        store
            .read_range("clip", 8..12)
            .await
            .expect("valid test fixture")
            .expect("valid test fixture"),
        b"ijkl"
    );
    discard(&root);
}
