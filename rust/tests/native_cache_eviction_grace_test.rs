mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{temp_directory, trusted_media_client, video_cache_key};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test(start_paused = true)]
async fn reclaims_an_unreferenced_blob_after_the_eviction_grace() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let directory = temp_directory("ghostr-cache-grace");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::with_eviction_grace(
        directory.clone(),
        10,
        used_bytes.clone(),
        Duration::from_secs(30),
    );
    tokio::time::resume();
    let cached = cache
        .download(&trusted_media_client(), &video_cache_key(), &url, None)
        .await
        .expect("cached video");
    tokio::time::pause();

    cache.retain(&HashSet::new()).await.expect("mark orphan");
    assert!(cached.path.exists());
    tokio::time::advance(Duration::from_secs(30)).await;
    cache.retain(&HashSet::new()).await.expect("reclaim orphan");

    assert!(!cached.path.exists());
    assert_eq!(*used_bytes.lock().await, 0);
    request.await.expect("upstream request");
    std::fs::remove_dir(directory).expect("remove cache");
}
