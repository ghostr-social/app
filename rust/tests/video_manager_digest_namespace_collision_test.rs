mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_video_manager};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn does_not_group_hashless_bytes_with_an_advertised_digest() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (hashless_url, hashless_request) = spawn_raw_server(response).await;
    let (digest_url, digest_request) = spawn_raw_server(response).await;
    let collision = format!("{:x}", Sha256::digest(hashless_url.as_bytes()));
    let hashless = EventBuilder::new(Kind::Custom(22), "Hashless")
        .tags([
            Tag::parse(["imeta", &format!("url {hashless_url}"), "m video/mp4"])
                .expect("hashless metadata"),
        ])
        .sign_with_keys(&Keys::generate())
        .expect("hashless event");
    let advertised = EventBuilder::new(Kind::Custom(22), "Advertised")
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {digest_url}"),
            format!("x {collision}"),
            "m video/mp4".to_owned(),
        ])
        .expect("digest metadata")])
        .sign_with_keys(&Keys::generate())
        .expect("digest event");
    let directory = temp_directory("ghostr-digest-namespace");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_native_videos(&hashless).remove(0))
        .await;
    videos
        .insert(canonical_native_videos(&advertised).remove(0))
        .await;
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 2);

    manager.synchronize_once().await.expect("synchronize");

    let values = downloads.lock().await;
    let hashless = values
        .values()
        .find(|video| video.nostr.expected_digest.is_none())
        .expect("hashless download");
    let advertised = values
        .values()
        .find(|video| video.nostr.expected_digest.is_some())
        .expect("advertised download");
    assert!(hashless.local_path().is_some());
    assert!(advertised.local_path().is_none());
    drop(values);
    hashless_request.await.expect("hashless request");
    digest_request.await.expect("digest request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
