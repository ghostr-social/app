mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_media_client, video_cache_key};
use tokio::sync::Mutex;

#[tokio::test]
async fn rejects_before_requesting_media_when_the_budget_is_full() {
    let directory = temp_directory("ghostr-full-cache");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let cache = NativeVideoCache::new(directory.clone(), 5, Arc::new(Mutex::new(5)));

    let result = cache
        .download(
            &trusted_media_client(),
            &video_cache_key(),
            "http://127.0.0.1:1/video.mp4",
            None,
        )
        .await;

    let error = match result {
        Ok(_) => panic!("full cache must reject the download"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("budget exhausted"));
    std::fs::remove_dir_all(directory).expect("remove cache");
}
