mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use support::fixtures::{temp_directory, video_id};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn stores_a_complete_video_and_accounts_for_its_bytes() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let directory = temp_directory("ghostr-cache-success");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());

    let cached = cache
        .download(&reqwest::Client::new(), &video_id(), &url)
        .await
        .expect("cached video");

    assert_eq!(cached.bytes, 5);
    assert_eq!(cached.content_length, Some(5));
    assert_eq!(
        tokio::fs::read(&cached.path).await.expect("video"),
        b"video"
    );
    assert_eq!(*used_bytes.lock().await, 5);
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
