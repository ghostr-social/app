//! Evicting a key clears its verification record, so bytes downloaded
//! later under the same key cannot inherit the old file's provenance.

mod store_fixture;

use ghostr_partial_store::partial_range_completion::Completion;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use store_fixture::temp_root;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_remove_clears_the_verification_record() {
    let root = temp_root("ghostr-partial-remove-verified");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());
    let digest = format!("{:x}", Sha256::digest(b"headtail"));
    store
        .write_range("clip", 0, b"headtail")
        .await
        .expect("bytes");
    store.set_total_len("clip", 8).await.expect("total length");
    store
        .finalize("clip", Some(digest.as_str()))
        .await
        .expect("verified finalize");

    store.remove("clip").await.expect("evict");

    assert_eq!(store.completion("clip").await.expect("completion"), None);
    assert_eq!(*used_bytes.lock().await, 0, "eviction releases the bytes");
    assert_eq!(
        std::fs::read_dir(&root).expect("store contents").count(),
        0,
        "eviction leaves nothing behind"
    );

    store
        .write_range("clip", 0, b"repeated")
        .await
        .expect("new bytes");
    store.set_total_len("clip", 8).await.expect("total length");
    store.finalize("clip", None).await.expect("plain finalize");

    assert_eq!(
        store.completion("clip").await.expect("completion"),
        Some(Completion::Unverified),
        "the evicted file's verification must not carry over"
    );
    assert_eq!(*used_bytes.lock().await, 8);
    std::fs::remove_dir_all(root).expect("remove store");
}
