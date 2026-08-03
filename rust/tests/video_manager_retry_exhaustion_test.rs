mod support;

use rust_lib_ghostr::video::event_index::new_native_video_index;
use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::new_native_downloads;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use support::fixtures::{canonical_video, temp_directory, trusted_video_manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::test(start_paused = true)]
async fn keeps_retrying_at_a_bounded_rate_after_eight_transient_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let calls = Arc::new(AtomicUsize::new(0));
    let server_calls = calls.clone();
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.expect("request");
            let attempt = server_calls.fetch_add(1, Ordering::SeqCst) + 1;
            let mut request = [0; 1024];
            let bytes = socket.read(&mut request).await.expect("read request");
            assert!(bytes > 0, "empty request");
            let response = if attempt == 9 {
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo".as_slice()
            } else {
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".as_slice()
            };
            socket.write_all(response).await.expect("write response");
        }
    });
    let directory = temp_directory("ghostr-retry-exhaustion");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_video(&format!("http://{address}/video.mp4")))
        .await;
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));
    let downloads = new_native_downloads();
    let manager = trusted_video_manager(downloads.clone(), cache, videos, 1);

    manager.synchronize_once().await.expect("initial attempt");
    for _ in 1..8 {
        tokio::time::advance(Duration::from_secs(60)).await;
        manager.synchronize_once().await.expect("retry boundary");
    }
    assert_eq!(calls.load(Ordering::SeqCst), 8);
    tokio::time::advance(Duration::from_secs(59)).await;
    manager.synchronize_once().await.expect("cooldown");
    assert_eq!(calls.load(Ordering::SeqCst), 8);
    tokio::time::advance(Duration::from_secs(1)).await;
    manager.synchronize_once().await.expect("late recovery");

    assert_eq!(calls.load(Ordering::SeqCst), 9);
    assert!(downloads
        .lock()
        .await
        .values()
        .all(|item| item.local_path().is_some()));
    server.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}
