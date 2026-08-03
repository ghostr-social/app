mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn hls_inventory_does_not_retain_an_orphaned_progressive_blob() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let progressive = video_event(Kind::Custom(22), 10, &url, &digest, "video/mp4");
    let hls = video_event(
        Kind::Custom(34236),
        20,
        "https://media.example/playlist.m3u8",
        &digest,
        "application/x-mpegurl",
    );
    let directory = temp_directory("ghostr-hls-retention");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = NativeVideoIndex::new(1);
    videos
        .insert(canonical_native_videos(&progressive).remove(0))
        .await;
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 5, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("cache progressive");
    let path = downloads
        .lock()
        .await
        .values()
        .next()
        .and_then(|video| video.local_path().map(|path| path.to_path_buf()))
        .expect("progressive path");

    videos.insert(canonical_native_videos(&hls).remove(0)).await;
    manager.synchronize_once().await.expect("replace with HLS");

    assert!(!path.exists());
    assert_eq!(*used_bytes.lock().await, 0);
    request.await.expect("progressive request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn video_event(
    kind: Kind,
    created_at: u64,
    url: &str,
    digest: &str,
    mime: &str,
) -> nostr_sdk::Event {
    EventBuilder::new(kind, "Video")
        .custom_created_at(Timestamp::from(created_at))
        .tags([
            Tag::parse(["d", "item"]).expect("identifier"),
            Tag::parse([
                "imeta".to_owned(),
                format!("url {url}"),
                format!("x {digest}"),
                format!("m {mime}"),
            ])
            .expect("video metadata"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("video event")
}
