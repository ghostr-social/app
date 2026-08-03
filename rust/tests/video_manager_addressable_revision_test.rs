mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::{new_native_downloads, NativeVideoDelivery};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn refreshes_state_and_metadata_for_each_addressable_revision() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let keys = Keys::generate();
    let progressive = video_event(&keys, 10, &url, &digest, "video/mp4");
    let hls = video_event(&keys, 20, &url, &digest, "application/vnd.apple.mpegurl");
    let directory = temp_directory("ghostr-addressable-revision");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos.record(&progressive).await;
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
        .and_then(|item| item.local_path().map(ToOwned::to_owned))
        .expect("progressive path");

    videos.record(&hls).await;
    manager
        .synchronize_once()
        .await
        .expect("apply HLS revision");

    let values = downloads.lock().await;
    let current = values.values().next().expect("current revision");
    assert_eq!(values.len(), 1);
    assert_eq!(current.event.created_at, 20);
    assert_eq!(current.nostr.delivery, NativeVideoDelivery::Hls);
    assert!(current.local_path().is_none());
    assert!(!path.exists());
    assert_eq!(*used_bytes.lock().await, 0);
    request.await.expect("progressive request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn video_event(keys: &Keys, at: u64, url: &str, digest: &str, mime: &str) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(34236), format!("revision {at}"))
        .custom_created_at(Timestamp::from(at))
        .tags([
            Tag::parse(["d", "clip"]).expect("identifier"),
            Tag::parse([
                "imeta".to_owned(),
                format!("url {url}"),
                format!("x {digest}"),
                format!("m {mime}"),
            ])
            .expect("video metadata"),
        ])
        .sign_with_keys(keys)
        .expect("signed video")
}
