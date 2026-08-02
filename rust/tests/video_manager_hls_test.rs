mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::video_manager::NativeVideoManager;
use std::sync::Arc;
use support::fixtures::temp_directory;
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn leaves_hls_remote_instead_of_caching_only_its_manifest() {
    let (url, request) = spawn_raw_server(
        b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nmanifest",
    )
    .await;
    let event = EventBuilder::new(Kind::Custom(34236), "Adaptive stream")
        .tags([
            Tag::parse(["d", "hls-video"]).expect("identifier"),
            Tag::parse(["imeta", &format!("url {url}"), "m application/x-mpegURL"])
                .expect("video tag"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let directory = temp_directory("ghostr-hls-manager");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_native_videos(&event).remove(0))
        .await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 32, Arc::new(Mutex::new(0)));
    let manager = NativeVideoManager::new(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("synchronize");

    let values = downloads.lock().await;
    let video = values.values().next().expect("HLS inventory item");
    assert!(!video.downloading);
    assert!(video.local_path.is_none());
    drop(values);
    assert_eq!(
        std::fs::read_dir(&directory)
            .expect("cache directory")
            .count(),
        0
    );
    request.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}
