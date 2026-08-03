mod support;

use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::{spawn_raw_server, spawn_response_sequence};
use tokio::sync::Mutex;

#[tokio::test(start_paused = true)]
async fn due_retry_keeps_recency_priority_over_an_older_fresh_download() {
    let unavailable = b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let video = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (newest_url, newest_requests) = spawn_response_sequence(vec![unavailable, video]).await;
    let (older_url, older_request) = spawn_raw_server(video).await;
    let directory = temp_directory("ghostr-recency-retry-priority");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = NativeVideoIndex::new(2);
    let mut newest = canonical_video(&newest_url);
    newest.inventory_id = "b".repeat(64);
    newest.coordinate = "newest-event".to_owned();
    newest.identity.event_id = "newest-event".to_owned();
    newest.identity.created_at += 1;
    newest.video.id = "b".repeat(64);
    videos.insert(newest).await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 5, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("transient failure");
    let mut older = canonical_video(&older_url);
    older.inventory_id = "c".repeat(64);
    older.coordinate = "older-event".to_owned();
    older.identity.event_id = "older-event".to_owned();
    older.video.id = "c".repeat(64);
    videos.insert(older).await;
    tokio::time::advance(Duration::from_secs(1)).await;

    manager.synchronize_once().await.expect("due retry");

    let values = downloads.lock().await;
    assert!(values[&"b".repeat(64)].local_path().is_some());
    assert!(values[&"c".repeat(64)].local_path().is_none());
    drop(values);
    assert!(!older_request.is_finished(), "older origin was requested");
    older_request.abort();
    newest_requests.await.expect("newest requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
