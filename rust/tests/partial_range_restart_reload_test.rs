mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_manifest_reloads_from_disk_after_a_restart() {
    let root = temp_directory("ghostr-partial-restart");
    {
        let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
        store.set_total_len("clip", 8).await.expect("total length");
        store.write_range("clip", 0, b"head").await.expect("head");
    }

    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());

    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        vec![0..4]
    );
    assert_eq!(
        store.missing_within("clip", 0..8).await.expect("missing"),
        vec![4..8]
    );
    assert!(!store.is_complete("clip").await.expect("completeness"));
    assert_eq!(*used_bytes.lock().await, 4);

    store.write_range("clip", 4, b"tail").await.expect("tail");
    assert!(store.is_complete("clip").await.expect("completeness"));
    assert_eq!(*used_bytes.lock().await, 8);
    std::fs::remove_dir_all(root).expect("remove store");
}
