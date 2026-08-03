mod support;

use rust_lib_ghostr::video::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use rust_lib_ghostr::video::native_models::NativeVideoCacheKey;
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_media_client};
use tokio::sync::Mutex;

#[tokio::test]
async fn rejects_an_unvalidated_cache_identifier_before_network_io() {
    let directory = temp_directory("ghostr-cache-id");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let cache = NativeVideoCache::new(directory.clone(), 10, Arc::new(Mutex::new(0)));

    let result = cache
        .download(
            &trusted_media_client(),
            &NativeVideoCacheKey::UrlDerived("not-a-sha256".to_owned()),
            "http://127.0.0.1:1/video.mp4",
            None,
        )
        .await;

    let error = match result {
        Ok(_) => panic!("invalid cache identifier was accepted"),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "native video cache identifier is invalid"
    );
    assert_eq!(std::fs::read_dir(&directory).expect("cache").count(), 0);
    std::fs::remove_dir(directory).expect("remove cache");
}
