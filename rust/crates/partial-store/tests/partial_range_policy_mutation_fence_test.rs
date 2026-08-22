mod store_fixture;

use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unresolved_policy_authority_fences_canonical_mutation() {
    let root = store_fixture::temp_root("policy-mutation-fence");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.write_range("clip", 0, b"abcdefghijkl").await.unwrap();
    store.set_total_len("clip", 16).await.unwrap();
    let manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .unwrap();
    let old_hash = format!("{:x}", Sha256::digest(&manifest));
    let intent = format!(
        r#"{{"version":2,"old_accounted":12,"new_accounted":8,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .unwrap();
    tokio::fs::create_dir(root.join("clip.part.evict"))
        .await
        .unwrap();
    drop(store);

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();

    assert_eq!(reopened.used_bytes().await, 12);
    assert_eq!(
        reopened.read_range("clip", 0..12).await.unwrap().unwrap(),
        b"abcdefghijkl"
    );
    assert!(reopened.write_range("clip", 12, b"mnop").await.is_err());
    assert_eq!(
        reopened.read_range("clip", 0..12).await.unwrap().unwrap(),
        b"abcdefghijkl"
    );
    assert_eq!(reopened.used_bytes().await, 12);

    tokio::fs::remove_dir(root.join("clip.part.evict"))
        .await
        .unwrap();
    reopened.write_range("clip", 12, b"mnop").await.unwrap();
    assert_eq!(reopened.used_bytes().await, 16);
    store_fixture::discard(&root);
}
