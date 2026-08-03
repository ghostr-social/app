mod support;

use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn reclaims_an_evicted_blob_for_the_replacement_video() {
    let (first_url, first_request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nfirst")
            .await;
    let (second_url, second_request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nother")
            .await;
    let directory = temp_directory("ghostr-manager-eviction");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = NativeVideoIndex::new(1);
    videos.insert(canonical_video(&first_url)).await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 5, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("cache first video");
    let first_path = downloads
        .lock()
        .await
        .values()
        .next()
        .and_then(|video| video.local_path().map(|path| path.to_path_buf()))
        .expect("first local path");
    let mut replacement = canonical_video(&second_url);
    replacement.inventory_id = "b".repeat(64);
    replacement.coordinate = "replacement-event".to_owned();
    replacement.identity.event_id = "replacement-event".to_owned();
    replacement.identity.created_at += 1;
    replacement.video.id = "b".repeat(64);
    videos.insert(replacement).await;

    manager.synchronize_once().await.expect("cache replacement");

    assert!(downloads
        .lock()
        .await
        .values()
        .all(|video| video.local_path().is_some()));
    assert!(!first_path.exists());
    assert_eq!(*used_bytes.lock().await, 5);
    first_request.await.expect("first upstream request");
    second_request.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}
