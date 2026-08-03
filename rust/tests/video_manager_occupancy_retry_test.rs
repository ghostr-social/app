mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test(start_paused = true)]
async fn retries_after_transient_cache_occupancy_is_released() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let directory = temp_directory("ghostr-occupancy-retry");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_video(&url)).await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(5));
    let cache = NativeVideoCache::new(directory.clone(), 5, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("occupied attempt");
    *used_bytes.lock().await = 0;
    tokio::time::advance(Duration::from_secs(1)).await;
    manager
        .synchronize_once()
        .await
        .expect("retry after release");

    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
