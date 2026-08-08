mod store_fixture;

use std::sync::Arc;
use store_fixture::{plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_writes_coalesce_overlaps_and_adjacency_and_account_bytes() {
    let root = temp_root("ghostr-partial-merge");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), used_bytes.clone());

    store.write_range("clip", 0, b"aaaa").await.expect("head");
    store.write_range("clip", 8, b"cccc").await.expect("tail");
    assert_eq!(
        store.present_ranges("clip").await.expect("split ranges"),
        vec![0..4, 8..12]
    );

    store.write_range("clip", 4, b"bbbb").await.expect("bridge");
    assert_eq!(
        store.present_ranges("clip").await.expect("merged ranges"),
        vec![0..12]
    );

    store.write_range("clip", 2, b"XX").await.expect("overlap");
    assert_eq!(
        store.present_ranges("clip").await.expect("stable ranges"),
        vec![0..12]
    );
    assert_eq!(*used_bytes.lock().await, 12);
    std::fs::remove_dir_all(root).expect("remove store");
}
