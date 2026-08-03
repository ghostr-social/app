mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn downloads_one_blob_for_distinct_posts_that_share_media() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let directory = temp_directory("ghostr-shared-manager");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let mut first = canonical_video(&url);
    first.video.id = digest.clone();
    first.video.expected_digest = Some(digest.clone());
    let mut second = canonical_video(&url);
    second.video.id = digest.clone();
    second.video.expected_digest = Some(digest);
    second.inventory_id = "b".repeat(64);
    second.coordinate = "second-event".to_owned();
    second.identity.event_id = "second-event".to_owned();
    videos.insert(first).await;
    videos.insert(second).await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 2);

    manager.synchronize_once().await.expect("synchronization");

    let values = downloads.lock().await;
    assert_eq!(values.len(), 2);
    assert!(values.values().all(|video| video.local_path().is_some()));
    request.await.expect("single upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
