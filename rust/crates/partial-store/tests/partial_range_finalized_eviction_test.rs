use crate::tests::store_fixture::{plain_store, temp_root};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn only_finalized_objects_are_indivisible_eviction_units() {
    let root = temp_root("ghostr-finalized-eviction");
    let used = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), std::sync::Arc::clone(&used));
    for key in ["finalized", "sparse"] {
        store
            .write_range(key, 0, b"abcdefghijkl")
            .await
            .expect("valid test fixture");
        store
            .set_total_len(key, 12)
            .await
            .expect("valid test fixture");
    }
    store
        .finalize("finalized", None)
        .await
        .expect("valid test fixture");

    let tail = 8..12;
    let finalized = store
        .evict_ranges("finalized", core::slice::from_ref(&tail))
        .await
        .expect("valid test fixture");
    let sparse = store
        .evict_ranges("sparse", core::slice::from_ref(&tail))
        .await
        .expect("valid test fixture");

    assert_eq!(finalized.freed_bytes(), 12);
    assert_eq!(finalized.ranges().len(), 1);
    assert_eq!(finalized.ranges()[0], 0..12);
    assert!(store
        .present_ranges("finalized")
        .await
        .expect("valid test fixture")
        .is_empty());
    assert_eq!(sparse.freed_bytes(), 4);
    assert_eq!(sparse.ranges().len(), 1);
    assert_eq!(sparse.ranges()[0], 8..12);
    assert_eq!(
        store
            .present_ranges("sparse")
            .await
            .expect("valid test fixture"),
        vec![0..8]
    );
    assert_eq!(
        store
            .read_range("sparse", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefgh".to_vec())
    );
    assert_eq!(*used.lock().await, 8);
    std::fs::remove_dir_all(root).expect("valid test fixture");
}
