mod store_fixture;

use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use store_fixture::temp_root;
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_finalize_rejects_and_discards_a_mismatched_digest() {
    let root = temp_root("ghostr-partial-mismatch");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = PartialRangeStore::new(root.clone(), used_bytes.clone());
    store
        .write_range("clip", 0, b"headtail")
        .await
        .expect("bytes");
    store.set_total_len("clip", 8).await.expect("total length");

    let advertised = "a".repeat(64);
    let error = store
        .finalize("clip", Some(advertised.as_str()))
        .await
        .expect_err("digest mismatch");

    assert!(error.to_string().contains("digest"), "unhelpful: {error}");
    assert_eq!(store.completion("clip").await.expect("completion"), None);
    assert_eq!(*used_bytes.lock().await, 0);
    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        Vec::<std::ops::Range<u64>>::new()
    );
    assert_eq!(
        std::fs::read_dir(&root).expect("store contents").count(),
        0,
        "mismatched bytes must not survive"
    );
    std::fs::remove_dir_all(root).expect("remove store");
}
