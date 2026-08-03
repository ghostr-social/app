mod support;

use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::{spawn_raw_server, unused_loopback_url};
use tokio::sync::Mutex;

#[tokio::test]
async fn newer_row_reuses_a_full_cache_blob_with_the_same_key() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (old_url, old_request) = spawn_raw_server(response).await;
    let directory = temp_directory("ghostr-shared-priority-blob");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = NativeVideoIndex::new(2);
    videos.insert(canonical_video(&old_url)).await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 5, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("cache old video");
    let mut newer = canonical_video(&unused_loopback_url().await);
    newer.inventory_id = "b".repeat(64);
    newer.coordinate = "new-event".to_owned();
    newer.identity.event_id = "new-event".to_owned();
    newer.identity.created_at += 1;
    videos.insert(newer).await;

    manager.synchronize_once().await.expect("reuse shared blob");

    let values = downloads.lock().await;
    assert!(values[&"a".repeat(64)].local_path().is_some());
    assert!(values[&"b".repeat(64)].local_path().is_some());
    assert_eq!(*used_bytes.lock().await, 5);
    drop(values);
    old_request.await.expect("old request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
