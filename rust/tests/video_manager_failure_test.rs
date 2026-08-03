mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager, video_id};
use support::http::unused_loopback_url;
use tokio::sync::Mutex;

#[tokio::test]
async fn marks_a_failed_native_download_as_no_longer_pending() {
    let directory = temp_directory("ghostr-manager-failure");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_video(&unused_loopback_url().await))
        .await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("synchronization");

    let values = downloads.lock().await;
    let download = values.get(&video_id()).expect("download");
    assert!(!download.is_downloading());
    assert!(download.local_path().is_none());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
