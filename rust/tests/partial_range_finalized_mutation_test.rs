mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn finalized_video_rejects_resize_and_more_bytes() {
    let root = temp_directory("ghostr-finalized-mutation");
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
    store.write_range("clip", 0, b"video").await.expect("bytes");
    store.set_total_len("clip", 5).await.expect("total");
    store.finalize("clip", None).await.expect("finalize");

    assert!(store.set_total_len("clip", 6).await.is_err());
    assert!(store.write_range("clip", 5, b"!").await.is_err());
    std::fs::remove_dir_all(root).expect("remove store");
}
