#![allow(dead_code)]

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn staged(name: &str) -> PathBuf {
    let root = super::store_fixture::temp_root(name);
    let store = super::store_fixture::plain_store(root.clone(), Arc::new(Mutex::new(0)));
    store.write_range("clip", 0, b"abcdefghijkl").await.unwrap();
    store.set_total_len("clip", 12).await.unwrap();
    drop(store);

    let old = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .unwrap();
    let new = retained_manifest();
    tokio::fs::write(root.join("clip.ranges.evict"), &new)
        .await
        .unwrap();
    tokio::fs::write(root.join("clip.evict.intent"), intent(&old, &new))
        .await
        .unwrap();
    root
}

pub async fn truncate(root: &Path) {
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(root.join("clip.part"))
        .await
        .unwrap();
    file.set_len(8).await.unwrap();
    file.sync_all().await.unwrap();
}

pub async fn reopen(root: &Path) -> ghostr_partial_store::partial_range_store::PartialRangeStore {
    let store = super::store_fixture::plain_store(root.to_owned(), Arc::new(Mutex::new(0)));
    store.load_existing().await.unwrap();
    store
}

pub fn assert_clean(root: &Path) {
    assert!(!root.join("clip.evict.intent").exists());
    assert!(!root.join("clip.ranges.evict").exists());
}

fn retained_manifest() -> Vec<u8> {
    let checksum = format!("{:x}", Sha256::digest(b"abcdefgh"));
    format!(
        r#"{{"version":2,"total_len":12,"intervals":[{{"start":0,"end":8,"sha256":"{checksum}"}}]}}"#
    )
    .into_bytes()
}

fn intent(old: &[u8], new: &[u8]) -> Vec<u8> {
    let old_hash = format!("{:x}", Sha256::digest(old));
    let new_hash = format!("{:x}", Sha256::digest(new));
    format!(
        r#"{{"version":3,"old_accounted":12,"new_accounted":8,"old_manifest_sha256":"{old_hash}","new_manifest_sha256":"{new_hash}","tail_end":8}}"#
    )
    .into_bytes()
}
