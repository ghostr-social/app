//! Most Nostr video notes carry no `imeta x`, so a byte-complete file
//! with nothing to check against must still leave the partial pool.

use crate::partial_range_completion::Completion;
use crate::tests::store_fixture::{plain_store, temp_root};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_finalize_promotes_a_complete_file_without_an_advertised_digest() {
    let root = temp_root("ghostr-partial-unadvertised");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), std::sync::Arc::clone(&used_bytes));
    store
        .write_range("clip", 0, b"headtail")
        .await
        .expect("bytes");
    store.set_total_len("clip", 8).await.expect("total length");

    let completion = store.finalize("clip", None).await.expect("finalize");
    let completed = root.join("clip.video");

    assert_eq!(completion, Completion::Unverified);
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
        2,
        "the completed file keeps its local checksum manifest"
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
