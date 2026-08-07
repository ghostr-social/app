//! Most Nostr video notes carry no `imeta x`, so a byte-complete file
//! with nothing to check against must still leave the partial pool.

mod store_fixture;

use ghostr_partial_store::partial_range_completion::Completion;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use store_fixture::temp_root;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_finalize_promotes_a_complete_file_without_an_advertised_digest() {
    let root = temp_root("ghostr-partial-unadvertised");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());
    store
        .write_range("clip", 0, b"headtail")
        .await
        .expect("bytes");
    store.set_total_len("clip", 8).await.expect("total length");

    let completed = store.finalize("clip", None).await.expect("finalize");

    assert_eq!(completed, store.completed_path("clip"));
    assert_eq!(
        tokio::fs::read(&completed).await.expect("completed bytes"),
        b"headtail"
    );
    let completion = store.completion("clip").await.expect("completion");
    assert_eq!(completion, Some(Completion::Unverified));
    assert!(
        !completion.expect("promoted").is_verified(),
        "nothing was checked, so nothing may claim verification"
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
        "the partial file and its manifest must be gone"
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
