mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::{new_native_downloads, NativeVideoDelivery};
use std::sync::Arc;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Notify};

#[tokio::test]
async fn in_flight_failure_does_not_restore_replaced_inventory() {
    let (url, requested, release, server) = blocked_failure_origin().await;
    let videos = new_native_video_index();
    let original = canonical_video(&url);
    let original_id = original.inventory_id.clone();
    videos.insert(original).await;
    let directory = temp_directory("ghostr-inflight-replacement");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let downloads = new_native_downloads();
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let manager = Arc::new(trusted_video_manager(
        downloads.clone(),
        cache,
        videos.clone(),
        1,
    ));
    let in_flight = tokio::spawn({
        let manager = manager.clone();
        async move { manager.synchronize_once().await }
    });
    requested.notified().await;

    let mut replacement = canonical_video("https://media.example/stream.m3u8");
    replacement.inventory_id = "b".repeat(64);
    replacement.identity.event_id = "replacement".to_owned();
    replacement.identity.created_at += 1;
    replacement.video.delivery = NativeVideoDelivery::Hls;
    videos.insert(replacement).await;
    manager.synchronize_once().await.expect("replace inventory");
    assert!(!downloads.lock().await.contains_key(&original_id));

    release.notify_one();
    in_flight
        .await
        .expect("download task")
        .expect("stale outcome");
    assert_eq!(downloads.lock().await.len(), 1);
    server.await.expect("failure origin");
    std::fs::remove_dir_all(directory).expect("remove cache");
}

async fn blocked_failure_origin() -> (
    String,
    Arc<Notify>,
    Arc<Notify>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let requested = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let server = tokio::spawn({
        let requested = requested.clone();
        let release = release.clone();
        async move {
            let (mut socket, _) = listener.accept().await.expect("request");
            let mut request = [0; 1024];
            let bytes = socket.read(&mut request).await.expect("read request");
            assert!(bytes > 0, "empty request");
            requested.notify_one();
            release.notified().await;
            socket
                .write_all(b"HTTP/1.1 500 Error\r\nContent-Length: 0\r\n\r\n")
                .await
                .expect("write response");
        }
    });
    (
        format!("http://{address}/video.mp4"),
        requested,
        release,
        server,
    )
}
