#![cfg(unix)]

mod store_fixture;

use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn impossible_policy_intent_cannot_inflate_cleanup_accounting() {
    let root = store_fixture::temp_root("policy-impossible-intent");
    let store = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.write_range("clip", 0, b"abcdefghijkl").await.unwrap();
    store.set_total_len("clip", 12).await.unwrap();
    let manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .unwrap();
    let old_hash = format!("{:x}", Sha256::digest(&manifest));
    tokio::fs::write(root.join("clip.part.evict"), b"abcdefgh")
        .await
        .unwrap();
    let intent = format!(
        r#"{{"version":2,"old_accounted":12,"new_accounted":16,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .unwrap();
    drop(store);
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o500)).unwrap();

    let reopened = store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.unwrap();

    assert_eq!(reopened.used_bytes().await, 20);
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    reopened.clear().await.unwrap();
    store_fixture::discard(&root);
}
