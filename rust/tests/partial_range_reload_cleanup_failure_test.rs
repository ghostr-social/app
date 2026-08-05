mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn reload_continues_when_an_unusable_entry_cannot_be_deleted() {
    let root = temp_directory("ghostr-reload-cleanup-failure");
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
