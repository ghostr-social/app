use super::support::{cached, temp_directory};
use crate::native_blob_store::NativeBlobStore;
use ghostr_media_model::native_models::NativeVideoCacheKey;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn invalid_active_native_blob_is_removed_and_reported() {
    let root = temp_directory("ghostr-invalid-blob");
    let path = root.join("clip.mp4");
    tokio::fs::create_dir(&path)
        .await
        .expect("blocking directory");
    let key = NativeVideoCacheKey::UrlDerived("b".repeat(64));
    let used = Arc::new(Mutex::new(5));
    let store = NativeBlobStore::new(used.clone(), Duration::ZERO);
    store.remember(key.clone(), cached(&path, 5)).await;

    let invalid = store
        .retain(&HashSet::from([key.clone()]))
        .await
        .expect("retain");

    assert_eq!(invalid, HashSet::from([key.clone()]));
    assert!(!path.exists());
    assert_eq!(*used.lock().await, 0);
    assert!(store.find(&key).await.expect("find").is_none());
    std::fs::remove_dir_all(root).expect("remove test directory");
}
