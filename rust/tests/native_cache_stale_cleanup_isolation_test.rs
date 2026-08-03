#![cfg(unix)]

mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{temp_directory, trusted_media_client, video_cache_key};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn stale_cleanup_failure_does_not_abort_cache_synchronization() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let directory = temp_directory("ghostr-stale-cleanup-isolation");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::with_eviction_grace(
        directory.clone(),
        10,
        used_bytes.clone(),
        Duration::ZERO,
    );
    cache
        .download(&trusted_media_client(), &video_cache_key(), &url, None)
        .await
        .expect("cached video");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o500))
        .expect("lock cache directory");

    let result = cache.retain(&HashSet::new()).await;

    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
        .expect("unlock cache directory");
    assert!(result.is_ok(), "stale cleanup leaked: {result:?}");
    assert_eq!(*used_bytes.lock().await, 5);
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
