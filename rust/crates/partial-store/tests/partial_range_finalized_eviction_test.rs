mod store_fixture;

use std::sync::Arc;
use store_fixture::{plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn only_finalized_objects_are_indivisible_eviction_units() {
    let root = temp_root("ghostr-finalized-eviction");
    let used = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), used.clone());
    for key in ["finalized", "sparse"] {
        store.write_range(key, 0, b"abcdefghijkl").await.unwrap();
        store.set_total_len(key, 12).await.unwrap();
    }
    store.finalize("finalized", None).await.unwrap();

    let tail = 8..12;
    let finalized = store
        .evict_ranges("finalized", std::slice::from_ref(&tail))
        .await
        .unwrap();
    let sparse = store
        .evict_ranges("sparse", std::slice::from_ref(&tail))
        .await
        .unwrap();

    assert_eq!(finalized.freed_bytes(), 12);
    assert_eq!(finalized.ranges().len(), 1);
    assert_eq!(finalized.ranges()[0], 0..12);
    assert!(store.present_ranges("finalized").await.unwrap().is_empty());
    assert_eq!(sparse.freed_bytes(), 4);
    assert_eq!(sparse.ranges().len(), 1);
    assert_eq!(sparse.ranges()[0], 8..12);
    assert_eq!(store.present_ranges("sparse").await.unwrap(), vec![0..8]);
    assert_eq!(
        store.read_range("sparse", 0..8).await.unwrap(),
        Some(b"abcdefgh".to_vec())
    );
    assert_eq!(*used.lock().await, 8);
    std::fs::remove_dir_all(root).unwrap();
}
