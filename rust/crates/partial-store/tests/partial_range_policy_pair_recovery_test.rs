use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unfinished_policy_pair_swap_restores_the_entire_old_object() {
    let root = crate::tests::store_fixture::temp_root("policy-pair-recovery");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");
    let old_manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .expect("valid test fixture");
    let new_manifest = retained_manifest();
    tokio::fs::rename(root.join("clip.part"), root.join("clip.part.evict.old"))
        .await
        .expect("valid test fixture");
    tokio::fs::rename(
        root.join("clip.ranges.json"),
        root.join("clip.ranges.evict.old"),
    )
    .await
    .expect("valid test fixture");
    tokio::fs::write(root.join("clip.part"), b"abcd\0\0\0\0ijkl")
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("clip.ranges.json"), &new_manifest)
        .await
        .expect("valid test fixture");
    write_intent(&root, &old_manifest).await;
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");

    assert_eq!(reopened.used_bytes().await, 12);
    assert_eq!(
        reopened
            .read_range("clip", 0..12)
            .await
            .expect("valid test fixture"),
        Some(b"abcdefghijkl".to_vec())
    );
    assert!(!root.join("clip.evict.intent").exists());
    assert!(!root.join("clip.part.evict.old").exists());
    assert!(!root.join("clip.ranges.evict.old").exists());
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

async fn write_intent(root: &std::path::Path, old: &[u8]) {
    let old_hash = format!("{:x}", Sha256::digest(old));
    let intent = format!(
        r#"{{"version":2,"old_accounted":12,"new_accounted":8,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .expect("valid test fixture");
}
