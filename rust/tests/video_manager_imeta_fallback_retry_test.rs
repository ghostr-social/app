mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_response_sequence;
use tokio::sync::Mutex;

#[tokio::test]
async fn retries_when_any_fallback_origin_failure_is_transient() {
    let evil = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nevil!";
    let unavailable = b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let video = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (primary, primary_requests) = spawn_response_sequence(vec![evil, evil]).await;
    let (fallback, fallback_requests) = spawn_response_sequence(vec![unavailable, video]).await;
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let event = EventBuilder::new(Kind::Custom(22), "clip")
        .tag(
            Tag::parse([
                "imeta".to_owned(),
                format!("url {primary}"),
                format!("fallback {fallback}"),
                format!("x {digest}"),
                "m video/mp4".to_owned(),
            ])
            .expect("video metadata"),
        )
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let videos = NativeVideoIndex::new(8);
    videos
        .insert(canonical_native_videos(&event).remove(0))
        .await;
    let downloads = new_native_downloads();
    let directory = temp_directory("ghostr-fallback-retry");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("first attempt");
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::time::resume();
    manager.synchronize_once().await.expect("retry");

    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    primary_requests.await.expect("primary requests");
    fallback_requests.await.expect("fallback requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
