use sha2::{Digest as _, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unresolved_policy_authority_fences_canonical_mutation() {
    let root = crate::tests::store_fixture::temp_root("policy-mutation-fence");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 16)
        .await
        .expect("valid test fixture");
    let manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .expect("valid test fixture");
    let old_hash = format!("{:x}", Sha256::digest(&manifest));
    let intent = format!(
        r#"{{"version":2,"old_accounted":12,"new_accounted":8,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .expect("valid test fixture");
    tokio::fs::create_dir(root.join("clip.part.evict"))
        .await
        .expect("valid test fixture");
    drop(store);

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");

    assert_eq!(reopened.used_bytes().await, 12);
    assert_eq!(
        reopened
            .read_range("clip", 0..12)
            .await
            .expect("valid test fixture")
            .expect("valid test fixture"),
        b"abcdefghijkl"
    );
    assert!(reopened.write_range("clip", 12, b"mnop").await.is_err());
    assert_eq!(
        reopened
            .read_range("clip", 0..12)
            .await
            .expect("valid test fixture")
            .expect("valid test fixture"),
        b"abcdefghijkl"
    );
    assert_eq!(reopened.used_bytes().await, 12);

    tokio::fs::remove_dir(root.join("clip.part.evict"))
        .await
        .expect("valid test fixture");
    reopened
        .write_range("clip", 12, b"mnop")
        .await
        .expect("valid test fixture");
    assert_eq!(reopened.used_bytes().await, 16);
    crate::tests::store_fixture::discard(&root);
}
