mod store_fixture;

use std::sync::Arc;
use store_fixture::{discard, plain_store, temp_root};
use tokio::sync::Mutex;

#[tokio::test]
async fn interrupted_policy_eviction_never_reloads_its_sparse_scratch_as_media() {
    let root = temp_root("ghostr-policy-eviction-restart");
    let manifest = root.join("clip.ranges.json");
    let store = plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store
        .write_range("clip", 0, b"abcdefghijkl")
        .await
        .expect("seed range");
    store.set_total_len("clip", 12).await.expect("total");
    let stable_manifest = tokio::fs::read(&manifest).await.expect("stable manifest");

    tokio::fs::remove_file(&manifest)
        .await
        .expect("hide manifest");
    tokio::fs::create_dir(&manifest)
        .await
        .expect("block manifest commit");
    store
        .evict_ranges("clip", std::slice::from_ref(&(4..8)))
        .await
        .expect_err("eviction commit must fail");
    tokio::fs::remove_dir(&manifest)
        .await
        .expect("unblock manifest");
    tokio::fs::write(&manifest, stable_manifest)
        .await
        .expect("restore stable manifest");
    drop(store);

    let reopened = plain_store(root.clone(), Arc::new(Mutex::new(0)));
    reopened.load_existing().await.expect("reload store");
    let middle = reopened.read_range("clip", 4..8).await.expect("read range");

    assert_eq!(middle.as_deref(), Some(b"efgh".as_slice()));
    discard(&root);
}
