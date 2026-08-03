use super::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use super::http::spawn_response_sequence;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub async fn assert_origin_response_is_retried(first: &'static [u8], prefix: &str) {
    let success = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, requests) = spawn_response_sequence(vec![first, success]).await;
    let directory = temp_directory(prefix);
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_video(&url)).await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("rejected response");
    tokio::time::advance(Duration::from_secs(1)).await;
    manager.synchronize_once().await.expect("retry response");

    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    requests.await.expect("upstream requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
