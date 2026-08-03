mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::video_manager::NativeVideoManager;
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn does_not_prefetch_a_private_network_media_url() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nvideo").await;
    let directory = temp_directory("ghostr-private-manager");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_video(&url)).await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = NativeVideoManager::new(downloads, cache, videos, 1).expect("manager");

    manager.synchronize_once().await.expect("synchronize");

    assert!(
        !request.is_finished(),
        "private endpoint received a request"
    );
    request.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}
