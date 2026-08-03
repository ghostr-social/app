mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::spawn_response_sequence;
use tokio::sync::Mutex;

#[tokio::test]
async fn refetches_an_active_blob_with_the_wrong_length() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, requests) = spawn_response_sequence(vec![response, response]).await;
    let directory = temp_directory("ghostr-manager-truncated-blob");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_video(&url)).await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);
    manager.synchronize_once().await.expect("initial download");
    let path = downloads
        .lock()
        .await
        .values()
        .next()
        .and_then(|item| item.local_path().map(ToOwned::to_owned))
        .expect("local path");
    tokio::fs::write(&path, b"bad")
        .await
        .expect("truncate blob");

    manager.synchronize_once().await.expect("reconcile cache");

    assert_eq!(
        tokio::fs::read(&path).await.expect("refetched blob"),
        b"video"
    );
    assert_eq!(*used_bytes.lock().await, 5);
    requests.await.expect("two upstream requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
