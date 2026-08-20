mod store_fixture;

use sha2::{Digest, Sha256};
use std::sync::Arc;
use store_fixture::{plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn partial_range_finalize_promotes_a_complete_file_when_the_digest_matches() {
    let root = temp_root("ghostr-partial-finalize");
    let used_bytes = Arc::new(Mutex::new(0));
    let store = plain_store(root.clone(), used_bytes.clone());
    store.write_range("clip", 4, b"tail").await.expect("tail");
    store.write_range("clip", 0, b"head").await.expect("head");
    store.set_total_len("clip", 8).await.expect("total length");
    let expected = format!("{:x}", Sha256::digest(b"headtail"));

    let completed = store
        .finalize("clip", Some(expected.as_str()))
        .await
        .expect("finalize");

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
    assert!(!root.join("clip.part").exists(), "the partial is gone");
    assert!(root.join("clip.ranges.json").exists(), "checksums remain");
    std::fs::remove_dir_all(root).expect("remove store");
}
