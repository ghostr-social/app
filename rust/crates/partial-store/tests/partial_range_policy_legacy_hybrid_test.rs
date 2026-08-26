use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn legacy_policy_hybrid_is_discarded_instead_of_undercounted() {
    let root = crate::tests::store_fixture::temp_root("policy-legacy-hybrid");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("clip.ranges.json"), retained_manifest())
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("clip.part.evict"), b"abcd\0\0\0\0ijkl")
        .await
        .expect("valid test fixture");
    tokio::fs::write(
        root.join("clip.evict.intent"),
        br#"{"version":1,"retained_bytes":8}"#,
    )
    .await
    .expect("valid test fixture");
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(reopened.used_bytes().await, 0);
    assert!(!root.join("clip.part").exists());
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
