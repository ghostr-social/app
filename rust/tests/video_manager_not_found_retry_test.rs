mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::spawn_response_sequence;
use tokio::sync::Mutex;

#[tokio::test(start_paused = true)]
async fn retries_media_that_has_not_reached_the_origin_yet() {
    let responses = vec![
        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".as_slice(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo".as_slice(),
    ];
    let (url, requests) = spawn_response_sequence(responses).await;
    let directory = temp_directory("ghostr-not-found-retry");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_video(&url)).await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("not found");
    tokio::time::advance(Duration::from_secs(1)).await;
    manager.synchronize_once().await.expect("propagated media");

    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    requests.await.expect("upstream requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
