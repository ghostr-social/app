mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use support::fixtures::{trusted_media_client, video_cache_key};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::test]
async fn releases_reserved_bytes_after_an_interrupted_download() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n12345")
            .await
            .expect("partial response");
    });
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("ghostr-transfer-{nonce}"));
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());

    let result = cache
        .download(
            &trusted_media_client(),
            &video_cache_key(),
            &format!("http://{address}/video.mp4"),
            None,
        )
        .await;

    assert!(result.is_err());
    assert_eq!(*used_bytes.lock().await, 0);
    assert_eq!(
        std::fs::read_dir(&directory).expect("read cache").count(),
        0
    );
    std::fs::remove_dir(&directory).expect("remove fixture");
}
