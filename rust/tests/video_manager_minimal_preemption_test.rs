mod support;

use rust_lib_ghostr::video::event_identity::CanonicalNativeVideo;
use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::{spawn_raw_server, spawn_response_sequence};
use tokio::sync::Mutex;

fn ranked_video(url: &str, id: char, created_at: u64) -> CanonicalNativeVideo {
    let mut item = canonical_video(url);
    item.inventory_id = id.to_string().repeat(64);
    item.coordinate = format!("{id}-event");
    item.identity.event_id = item.coordinate.clone();
    item.identity.created_at = created_at;
    item.video.id = item.inventory_id.clone();
    item
}

#[tokio::test(start_paused = true)]
async fn preemption_evicts_only_the_lower_ranked_bytes_that_are_needed() {
    let two = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\naa";
    let three = b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nbbb";
    let (oldest_url, oldest_request) = spawn_raw_server(two).await;
    let (middle_url, middle_request) = spawn_raw_server(three).await;
    let (newest_url, newest_requests) = spawn_response_sequence(vec![two, two]).await;
    let directory = temp_directory("ghostr-minimal-preemption");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = NativeVideoIndex::new(3);
    videos.insert(ranked_video(&oldest_url, 'a', 10)).await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 6, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("cache oldest");
    videos.insert(ranked_video(&middle_url, 'b', 20)).await;
    manager.synchronize_once().await.expect("cache middle");
    videos.insert(ranked_video(&newest_url, 'c', 30)).await;
    manager.synchronize_once().await.expect("record shortfall");
    tokio::time::advance(Duration::from_secs(1)).await;

    manager.synchronize_once().await.expect("preempt and retry");

    let values = downloads.lock().await;
    assert!(values[&"a".repeat(64)].local_path().is_none());
    assert!(values[&"b".repeat(64)].local_path().is_some());
    assert!(values[&"c".repeat(64)].local_path().is_some());
    assert_eq!(*used_bytes.lock().await, 5);
    drop(values);
    oldest_request.await.expect("oldest request");
    middle_request.await.expect("middle request");
    newest_requests.await.expect("newest requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
