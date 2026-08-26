use crate::tests::store_fixture::{plain_store, temp_root};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_store_quarantines_an_unreadable_manifest_entry() {
    let root = temp_root("ghostr-manifest-read-failure");
    std::fs::create_dir_all(&root).expect("create store root");
    std::fs::create_dir(root.join("clip.ranges.json")).expect("blocking manifest directory");
    let store = plain_store(root.clone(), Arc::new(Mutex::new(0)));

    store
        .load_existing()
        .await
        .expect("one corrupt entry does not sink startup");
    assert!(store
        .present_ranges("clip")
        .await
        .expect("quarantined entry")
        .is_empty());
    assert_eq!(
        store
            .read_range("clip", 0..1)
            .await
            .expect("valid test fixture"),
        None
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
