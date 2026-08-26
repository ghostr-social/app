#![cfg(unix)]

use sha2::{Digest as _, Sha256};
use std::os::unix::fs::PermissionsExt as _;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn committed_policy_corruption_charges_every_quarantined_payload() {
    let root = crate::tests::store_fixture::temp_root("policy-committed-corruption");
    let store = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("valid test fixture");
    store
        .set_total_len("clip", 12)
        .await
        .expect("valid test fixture");
    tokio::fs::rename(root.join("clip.part"), root.join("clip.part.evict.old"))
        .await
        .expect("valid test fixture");
    tokio::fs::rename(
        root.join("clip.ranges.json"),
        root.join("clip.ranges.evict.old"),
    )
    .await
    .expect("valid test fixture");
    tokio::fs::write(
        root.join("clip.ranges.evict.old"),
        underclaimed_backup_manifest(),
    )
    .await
    .expect("valid test fixture");
    tokio::fs::write(root.join("clip.part"), b"xbcd\0\0\0\0ijkl")
        .await
        .expect("valid test fixture");
    tokio::fs::write(root.join("clip.ranges.json"), retained_manifest())
        .await
        .expect("valid test fixture");
    drop(store);
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o500))
        .expect("valid test fixture");

    let reopened = crate::tests::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("valid test fixture");

    assert_eq!(
        reopened
            .read_range("clip", 0..4)
            .await
            .expect("valid test fixture"),
        None
    );
    assert_eq!(reopened.used_bytes().await, 24);
    assert!(root.join("clip.part").exists());
    assert!(root.join("clip.part.evict.old").exists());
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
        .expect("valid test fixture");
    reopened.clear().await.expect("valid test fixture");
    assert_eq!(reopened.used_bytes().await, 0);
    crate::tests::store_fixture::discard(&root);
}

fn underclaimed_backup_manifest() -> Vec<u8> {
    let first = format!("{:x}", Sha256::digest(b"a"));
    format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":1,"sha256":"{first}"}}]}}"#
    )
    .into_bytes()
}

fn retained_manifest() -> Vec<u8> {
    let first = format!("{:x}", Sha256::digest(b"abcd"));
    let last = format!("{:x}", Sha256::digest(b"ijkl"));
    format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":4,"sha256":"{first}"}},{{"start":8,"end":12,"sha256":"{last}"}}]}}"#
    )
    .into_bytes()
}
