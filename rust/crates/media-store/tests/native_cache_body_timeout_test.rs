mod cache_fixture;

use cache_fixture::{temp_directory, video_cache_key};
use ghostr_media_store::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use ghostr_net::outbound_media_client::{MediaHttpClient, MediaHttpTimeouts};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::test]
async fn abandons_a_response_body_that_stops_making_progress() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n12345")
            .await
            .expect("partial response");
        std::future::pending::<()>().await;
    });
    let timeouts = MediaHttpTimeouts::new(Duration::from_secs(1), Duration::from_millis(100))
        .expect("timeouts");
    let client = MediaHttpClient::trusted_with_timeouts(timeouts).expect("media client");
    let directory = temp_directory("ghostr-body-timeout");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());

    let result = tokio::time::timeout(
        Duration::from_secs(1),
        cache.download(
            &client,
            &video_cache_key(),
            &format!("http://{address}/video.mp4"),
            None,
        ),
    )
    .await
    .expect("bounded download");

    assert!(result.is_err());
    assert_eq!(*used_bytes.lock().await, 0);
    assert_eq!(std::fs::read_dir(&directory).expect("cache").count(), 0);
    server.abort();
    std::fs::remove_dir(directory).expect("remove cache");
}
