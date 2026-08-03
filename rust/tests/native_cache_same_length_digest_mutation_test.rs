mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::NativeVideoIndex;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_response_sequence;
use tokio::sync::Mutex;

#[tokio::test]
async fn replaces_a_same_length_blob_that_no_longer_matches_its_digest() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, requests) = spawn_response_sequence(vec![response, response]).await;
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let media = Tag::parse([
        "imeta".to_owned(),
        format!("url {url}"),
        format!("x {digest}"),
        "m video/mp4".to_owned(),
    ])
    .expect("video metadata");
    let event = EventBuilder::new(Kind::Custom(22), "clip")
        .tag(media)
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let videos = NativeVideoIndex::new(8);
    videos
        .insert(canonical_native_videos(&event).remove(0))
        .await;
    let downloads = new_native_downloads();
    let directory = temp_directory("ghostr-same-length-mutation");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);
    manager
        .synchronize_once()
        .await
        .expect("first synchronization");
    let path = downloads
        .lock()
        .await
        .values()
        .next()
        .and_then(|item| item.local_path().map(ToOwned::to_owned))
        .expect("cached path");
    std::fs::write(&path, b"evil!").expect("mutate blob");
    std::fs::File::open(&path)
        .expect("open blob")
        .set_modified(SystemTime::now() + Duration::from_secs(5))
        .expect("change modification time");

    manager
        .synchronize_once()
        .await
        .expect("repair synchronization");

    assert_eq!(std::fs::read(&path).expect("repaired blob"), b"video");
    requests.await.expect("origin requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
