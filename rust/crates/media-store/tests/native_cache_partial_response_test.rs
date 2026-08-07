mod cache_fixture;

use cache_fixture::raw_http::spawn_raw_server;
use cache_fixture::{media_client, temp_directory, video_cache_key};
use ghostr_media_store::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn rejects_an_unsolicited_partial_response() {
    let response = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\n\r\nvid";
    let (url, request) = spawn_raw_server(response).await;
    let directory = temp_directory("ghostr-cache-partial-response");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());

    let result = cache
        .download(&media_client(), &video_cache_key(), &url, None)
        .await;

    assert!(result.is_err());
    assert_eq!(*used_bytes.lock().await, 0);
    assert_eq!(std::fs::read_dir(&directory).expect("cache").count(), 0);
    request.await.expect("upstream request");
    std::fs::remove_dir(directory).expect("remove cache");
}
