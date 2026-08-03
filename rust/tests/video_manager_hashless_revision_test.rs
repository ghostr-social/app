mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_response_sequence;
use tokio::sync::Mutex;

#[tokio::test]
async fn refetches_a_hashless_mutable_url_for_each_addressable_revision() {
    let old = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nold!!";
    let new = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nnew!!";
    let (url, requests) = spawn_response_sequence(vec![old, new]).await;
    let keys = Keys::generate();
    let videos = new_native_video_index();
    videos.record(&video_event(&keys, 10, &url)).await;
    let directory = temp_directory("ghostr-hashless-revision");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let downloads = new_native_downloads();
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 5, used_bytes.clone());
    let manager = trusted_video_manager(downloads.clone(), cache, videos.clone(), 1);
    manager.synchronize_once().await.expect("first revision");

    videos.record(&video_event(&keys, 20, &url)).await;
    manager.synchronize_once().await.expect("second revision");

    let values = downloads.lock().await;
    let current = values.values().next().expect("current revision");
    let path = current.local_path().expect("local path");
    assert_eq!(current.event.created_at, 20);
    assert_eq!(tokio::fs::read(path).await.expect("cached bytes"), b"new!!");
    assert_eq!(*used_bytes.lock().await, 5);
    drop(values);
    requests.await.expect("revision requests");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn video_event(keys: &Keys, at: u64, url: &str) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(34236), format!("revision {at}"))
        .custom_created_at(Timestamp::from(at))
        .tags([
            Tag::parse(["d", "clip"]).expect("identifier"),
            Tag::parse(["imeta", &format!("url {url}"), "m video/mp4"]).expect("video metadata"),
        ])
        .sign_with_keys(keys)
        .expect("signed video")
}
