mod store_fixture;

use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use store_fixture::temp_root;
use tokio::sync::Mutex;

#[tokio::test]
async fn reload_continues_when_an_unusable_entry_cannot_be_deleted() {
    let root = temp_root("ghostr-reload-cleanup-failure");
    std::fs::create_dir_all(root.join("blocked.part")).expect("blocking directory");
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));

    store
        .load_existing()
        .await
        .expect("startup remains available");

    assert!(
        root.join("blocked.part").is_dir(),
        "failed cleanup is isolated"
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
