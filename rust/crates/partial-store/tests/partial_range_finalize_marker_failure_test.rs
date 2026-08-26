use crate::tests::store_fixture::{plain_store, temp_root};
use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn failed_completion_marker_keeps_the_partial_readable_and_retryable() {
    let root = temp_root("ghostr-finalize-marker-failure");
    let store = plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"headtail")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 8)
        .await
        .expect("valid test fixture");
    let expected = format!("{:x}", Sha256::digest(b"headtail"));
    std::fs::create_dir(root.join("clip.verified")).expect("valid test fixture");

    assert!(store.finalize("clip", Some(&expected)).await.is_err());
    assert!(root.join("clip.part").is_file());
    assert!(!root.join("clip.video").exists());
    assert_eq!(
        store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"headtail".to_vec())
    );

    std::fs::remove_dir(root.join("clip.verified")).expect("valid test fixture");
    store
        .finalize("clip", Some(&expected))
        .await
        .expect("valid test fixture");
    assert_eq!(
        store
            .read_range("clip", 0..8)
            .await
            .expect("valid test fixture"),
        Some(b"headtail".to_vec())
    );
    std::fs::remove_dir_all(root).expect("valid test fixture");
}
