mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_reads_refuse_spans_that_cross_holes() {
    let root = temp_directory("ghostr-partial-holes");
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));

    store.write_range("clip", 0, b"aaaa").await.expect("head");
    store.write_range("clip", 8, b"cccc").await.expect("tail");

    assert_eq!(
        store.read_range("clip", 0..4).await.expect("present head"),
        Some(b"aaaa".to_vec())
    );
    assert_eq!(
        store.read_range("clip", 9..11).await.expect("inner tail"),
        Some(b"cc".to_vec())
    );
    assert_eq!(
        store
            .read_range("clip", 0..12)
            .await
            .expect("span with hole"),
        None
    );
    assert_eq!(
        store
            .read_range("clip", 2..6)
            .await
            .expect("edge into hole"),
        None
    );
    assert_eq!(
        store.read_range("unknown", 0..1).await.expect("absent key"),
        None
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
