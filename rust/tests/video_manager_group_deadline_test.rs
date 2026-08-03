mod support;

use nostr_sdk::{EventBuilder, Keys, Kind, Tag, Timestamp};
use rust_lib_ghostr::video::event_identity::canonical_native_videos;
use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use rust_lib_ghostr::video::video_manager::{NativeVideoManager, NativeVideoManagerConfiguration};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::temp_directory;
use support::http::spawn_raw_server;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::test]
async fn bounds_the_total_time_spent_on_a_candidate_group() {
    let (stalled_url, stalled) = stalled_origin().await;
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (good_url, good_request) = spawn_raw_server(response).await;
    let digest = format!("{:x}", Sha256::digest(b"video"));
    let videos = new_native_video_index();
    for (created_at, url) in [(20, &stalled_url), (10, &good_url)] {
        videos
            .insert(canonical_native_videos(&video_event(url, &digest, created_at)).remove(0))
            .await;
    }
    let directory = temp_directory("ghostr-group-deadline");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let configuration =
        NativeVideoManagerConfiguration::new(MediaHttpClient::trusted().expect("client"), 1)
            .with_candidate_policy(2, Duration::from_millis(100));
    let manager =
        NativeVideoManager::with_configuration(downloads.clone(), cache, videos, configuration);
    let started = tokio::time::Instant::now();

    tokio::time::timeout(Duration::from_millis(500), manager.synchronize_once())
        .await
        .expect("group deadline")
        .expect("bounded attempt");

    assert!(started.elapsed() < Duration::from_millis(500));
    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_none()));
    assert!(!good_request.is_finished(), "later candidate was attempted");
    manager.synchronize_once().await.expect("remaining mirror");
    let values = downloads.lock().await;
    assert!(
        values.values().all(|item| item.local_path().is_some()),
        "{values:#?}"
    );
    drop(values);
    good_request.await.expect("good request");
    stalled.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}

async fn stalled_origin() -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        let bytes = socket.read(&mut request).await.expect("read request");
        assert!(bytes > 0, "empty request");
        std::future::pending::<()>().await;
        drop(socket);
    });
    (format!("http://{address}/video.mp4"), task)
}

fn video_event(url: &str, digest: &str, created_at: u64) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(22), url)
        .custom_created_at(Timestamp::from(created_at))
        .tags([Tag::parse([
            "imeta".to_owned(),
            format!("url {url}"),
            format!("x {digest}"),
            "m video/mp4".to_owned(),
        ])
        .expect("metadata")])
        .sign_with_keys(&Keys::generate())
        .expect("signed video")
}
