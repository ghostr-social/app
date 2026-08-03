mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn tries_another_origin_after_a_digest_mismatch() {
    let (bad_url, bad_request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nevil!")
            .await;
    let (good_url, good_request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let bad = video_event(&bad_url, &digest, "Bad mirror", 20);
    let good = video_event(&good_url, &digest, "Good mirror", 10);
    let directory = temp_directory("ghostr-digest-mirror");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.insert(canonical_native_videos(&bad).remove(0)).await;
    videos
        .insert(canonical_native_videos(&good).remove(0))
        .await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 2);

    manager.synchronize_once().await.expect("synchronize");

    let values = downloads.lock().await;
    assert!(values.values().all(|video| video.local_path().is_some()));
    let paths = values
        .values()
        .filter_map(|video| video.local_path().map(|path| path.to_path_buf()))
        .collect::<HashSet<_>>();
    assert_eq!(paths.len(), 1);
    let path = paths.into_iter().next().expect("cached path");
    drop(values);
    assert_eq!(tokio::fs::read(path).await.expect("cached video"), b"video");
    bad_request.await.expect("bad request");
    good_request.await.expect("good request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn video_event(url: &str, digest: &str, content: &str, created_at: u64) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), content)
        .custom_created_at(Timestamp::from(created_at))
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {url}"),
            format!("x {digest}"),
            "m video/mp4".to_owned(),
        ])
        .expect("video metadata")])
        .sign_with_keys(&Keys::generate())
        .expect("video event")
}
