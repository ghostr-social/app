mod store_fixture;

use std::sync::Arc;
use store_fixture::{plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn empty_partial_write_changes_nothing() {
    let root = temp_root("ghostr-empty-range-write");
    let store = plain_store(root.clone(), Arc::new(Mutex::new(0)));

    store
        .write_range("clip", 4, &[])
        .await
        .expect("empty write");

    assert!(store
        .present_ranges("clip")
        .await
        .expect("ranges")
        .is_empty());
    assert!(!root.join("clip.part").exists());
}
