mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use support::fixtures::{
    temp_directory, trusted_media_client, video_cache_file_id, video_cache_key,
};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn reports_the_original_and_partial_cleanup_failures() {
    let (url, request) =
        spawn_raw_server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo")
            .await;
    let directory = temp_directory("ghostr-cleanup-failure");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let partial = directory.join(format!("{}.partial", video_cache_file_id()));
    std::fs::create_dir(&partial).expect("blocking partial directory");
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));

    let error = match cache
        .download(&trusted_media_client(), &video_cache_key(), &url, None)
        .await
    {
        Ok(_) => panic!("download must fail"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("partial-file cleanup also failed"));
    request.await.expect("upstream request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
