use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unusable_legacy_authority_outlives_failed_canonical_cleanup() {
    let root = crate::tests::store_fixture::temp_root("policy-legacy-authority-order");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    tokio::fs::create_dir_all(&root)
        .await
        .expect("valid test fixture");
    tokio::fs::create_dir(root.join("clip.part"))
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("clip.ranges.json"), retained_manifest())
        .await
        .expect("valid test fixture");
    tokio::fs::write(
        root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .expect("valid test fixture");

    store.load_existing().await.expect("valid test fixture");

    assert!(root.join("clip.evict.intent").exists());
    assert_eq!(
        store
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    tokio::fs::remove_dir(root.join("clip.part"))
        .await
        .expect("valid test fixture");
    store.clear().await.expect("valid test fixture");
    assert!(!root.join("clip.evict.intent").exists());
    crate::tests::store_fixture::discard(&root);
}

fn retained_manifest() -> Vec<u8> {
    let first = format!("{:x}", Sha256::digest(b"abcd"));
    let last = format!("{:x}", Sha256::digest(b"ijkl"));
    format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":4,"sha256":"{first}"}},{{"start":8,"end":12,"sha256":"{last}"}}]}}"#
    )
    .into_bytes()
}
