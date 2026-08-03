#![cfg(unix)]

mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use support::fixtures::{
    temp_directory, trusted_media_client, video_cache_file_id, video_cache_key,
};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn keeps_an_unremovable_interrupted_partial_charged_until_reclaimed() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let directory = temp_directory("ghostr-interrupted-reclamation");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let partial = directory.join(format!("{}.partial", video_cache_file_id()));
    std::fs::write(&partial, []).expect("create partial");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o555))
        .expect("block cleanup");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());

    let result = cache
        .download(&trusted_media_client(), &video_cache_key(), &url, None)
        .await;

    assert!(result.is_err());
    assert_eq!(
        tokio::fs::metadata(&partial).await.expect("partial").len(),
        5
    );
    assert_eq!(*used_bytes.lock().await, 5);
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o755))
        .expect("allow cleanup");
    cache
        .retain(&HashSet::new())
        .await
        .expect("reclaim partial");
    assert!(!partial.exists());
    assert_eq!(*used_bytes.lock().await, 0);
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
