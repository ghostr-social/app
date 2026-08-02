mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use support::fixtures::{temp_directory, video_id};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn releases_reserved_bytes_when_a_completed_file_cannot_be_installed() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let directory = temp_directory("ghostr-cache-rename");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let completed = directory.join(format!("{}.mp4", video_id()));
    std::fs::create_dir(&completed).expect("blocking directory");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());

    let result = cache
        .download(&reqwest::Client::new(), &video_id(), &url)
        .await;

    assert!(result.is_err());
    assert_eq!(*used_bytes.lock().await, 0);
    assert!(!directory.join(format!("{}.partial", video_id())).exists());
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
