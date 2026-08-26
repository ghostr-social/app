use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn malformed_policy_authority_never_adopts_an_ambiguous_hybrid() {
    let root = crate::tests::store_fixture::temp_root("policy-malformed-authority");
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
    tokio::fs::write(root.join("clip.evict.intent"), b"{malformed")
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
