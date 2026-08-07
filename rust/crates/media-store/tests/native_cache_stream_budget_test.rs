mod cache_fixture;

use cache_fixture::raw_http::spawn_raw_server;
use cache_fixture::{media_client, temp_directory, video_cache_key};
use ghostr_media_store::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn enforces_the_budget_for_a_stream_without_content_length() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nvideo").await;
    let directory = temp_directory("ghostr-stream-budget");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 4, used_bytes.clone());

    let result = cache
        .download(&media_client(), &video_cache_key(), &url, None)
        .await;

    let error = match result {
        Ok(_) => panic!("oversized stream was accepted"),
        Err(error) => error,
    };
    assert_eq!(error.to_string(), "native video exceeds cache capacity");
    assert_eq!(*used_bytes.lock().await, 0);
    assert_eq!(std::fs::read_dir(&directory).expect("cache").count(), 0);
    request.await.expect("upstream request");
    std::fs::remove_dir(directory).expect("remove cache");
}
