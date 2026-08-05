mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_store_reports_an_unreadable_manifest_entry() {
    let root = temp_directory("ghostr-manifest-read-failure");
    std::fs::create_dir_all(&root).expect("create store root");
    std::fs::create_dir(root.join("clip.ranges.json")).expect("blocking manifest directory");
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));

    store
        .load_existing()
        .await
        .expect("one corrupt entry does not sink startup");
    let error = store
        .present_ranges("clip")
        .await
        .expect_err("directory cannot be read as a manifest");

    assert!(error.to_string().contains("read partial range manifest"));
    std::fs::remove_dir_all(root).expect("remove store");
}
