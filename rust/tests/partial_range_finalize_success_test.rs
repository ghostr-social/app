mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_finalize_promotes_a_complete_file_when_the_digest_matches() {
    let root = temp_directory("ghostr-partial-finalize");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());
    store.write_range("clip", 4, b"tail").await.expect("tail");
    store.write_range("clip", 0, b"head").await.expect("head");
    store.set_total_len("clip", 8).await.expect("total length");
    let expected = format!("{:x}", Sha256::digest(b"headtail"));

    let completed = store.finalize("clip", &expected).await.expect("finalize");

    assert_eq!(
        tokio::fs::read(&completed).await.expect("completed bytes"),
        b"headtail"
    );
    assert!(store.is_complete("clip").await.expect("completeness"));
    assert_eq!(
        store.read_range("clip", 4..8).await.expect("read"),
        Some(b"tail".to_vec())
    );
    assert_eq!(*used_bytes.lock().await, 8);
    assert_eq!(
        std::fs::read_dir(&root).expect("store contents").count(),
        1,
        "only the completed file should remain"
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
