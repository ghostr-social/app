use super::support::{cached, temp_directory};
use crate::native_blob_store::NativeBlobStore;
use ghostr_media_model::native_models::NativeVideoCacheKey;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn active_native_blob_is_revalidated_retained_and_reusable() {
    let root = temp_directory("ghostr-active-blob");
    let path = root.join("clip.mp4");
    tokio::fs::write(&path, b"video").await.expect("write blob");
    let key = NativeVideoCacheKey::UrlDerived("a".repeat(64));
    let used = Arc::new(Mutex::new(5));
    let store = NativeBlobStore::new(used.clone(), Duration::ZERO);
    store.remember(key.clone(), cached(&path, 5)).await;

    let invalid = store
        .retain(&HashSet::from([key.clone()]))
        .await
        .expect("retain");
    let found = store.find(&key).await.expect("find").expect("cached blob");

    assert!(invalid.is_empty());
    assert_eq!(found.path, path);
    assert_eq!(*used.lock().await, 5);
    std::fs::remove_dir_all(root).expect("remove test directory");
}
