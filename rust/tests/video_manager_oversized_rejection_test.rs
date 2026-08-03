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
async fn does_not_retry_an_object_larger_than_the_entire_cache() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let calls = Arc::new(AtomicUsize::new(0));
    let server_calls = calls.clone();
    let server = tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.expect("request");
            server_calls.fetch_add(1, Ordering::SeqCst);
            let mut request = [0; 1024];
            let bytes = socket.read(&mut request).await.expect("read request");
            assert!(bytes > 0, "empty request");
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nvideo")
                .await
                .expect("write response");
        }
    });
    let directory = temp_directory("ghostr-manager-oversized");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let videos = new_native_video_index();
    videos
        .insert(canonical_video(&format!("http://{address}/video.mp4")))
        .await;
    let cache = NativeVideoCache::new(directory.clone(), 4, Arc::new(Mutex::new(0)));
    let manager = trusted_video_manager(new_native_downloads(), cache, videos, 1);

    manager.synchronize_once().await.expect("first attempt");
    tokio::time::advance(Duration::from_secs(60)).await;
    manager.synchronize_once().await.expect("retry boundary");

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    server.abort();
    std::fs::remove_dir_all(directory).expect("remove cache");
}
