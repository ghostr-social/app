mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn rejects_bytes_that_do_not_match_the_advertised_digest() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let event = EventBuilder::new(Kind::Custom(22), "Untrusted origin")
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {url}"),
            format!("x {}", "a".repeat(64)),
            "m video/mp4".to_owned(),
        ])
        .expect("video tag")])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let directory = temp_directory("ghostr-digest-mismatch");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_native_videos(&event).remove(0))
        .await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("synchronization");

    assert!(downloads
        .lock()
        .await
        .values()
        .all(|video| video.local_path().is_none()));
    assert_eq!(*used_bytes.lock().await, 0);
    assert_eq!(std::fs::read_dir(&directory).expect("cache").count(), 0);
    request.await.expect("upstream request");
    std::fs::remove_dir(directory).expect("remove cache");
}
