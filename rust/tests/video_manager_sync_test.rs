mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager, video_id};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn downloads_each_discovered_video_identity_only_once() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let directory = temp_directory("ghostr-manager");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_video(&url)).await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager
        .synchronize_once()
        .await
        .expect("first synchronization");
    manager
        .synchronize_once()
        .await
        .expect("second synchronization");

    let values = downloads.lock().await;
    let download = values.get(&video_id()).expect("download");
    assert!(!download.is_downloading());
    assert!(download.local_path().is_some_and(|path| path.exists()));
    assert_eq!(values.len(), 1);
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
