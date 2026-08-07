mod cache_fixture;

use cache_fixture::raw_http::spawn_raw_server;
use cache_fixture::{media_client, temp_directory, video_id};
use ghostr_media_model::native_models::NativeVideoCacheKey;
use ghostr_media_store::native_cache::{prepare_native_cache_directory, NativeVideoCache};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn does_not_reuse_a_hashless_blob_for_an_advertised_digest() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (hashless_url, hashless_request) = spawn_raw_server(response).await;
    let (digest_url, digest_request) = spawn_raw_server(response).await;
    let directory = temp_directory("ghostr-cache-digest-namespace");
    prepare_native_cache_directory(&directory).expect("prepare cache");
    let used_bytes = Arc::new(Mutex::new(0));
    let cache = NativeVideoCache::new(directory.clone(), 10, used_bytes.clone());
    let collision = video_id();
    let hashless_key = NativeVideoCacheKey::UrlDerived(collision.clone());
    let digest_key = NativeVideoCacheKey::AdvertisedDigest(collision.clone());

    let cached = cache
        .download(&media_client(), &hashless_key, &hashless_url, None)
        .await
        .expect("hashless cache entry");
    let advertised = cache
        .download(&media_client(), &digest_key, &digest_url, Some(&collision))
        .await;

    assert!(advertised.is_err());
    assert!(cached.path.exists());
    let file_id = cached
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .expect("cache file identifier");
    assert_eq!(file_id.len(), 64);
    assert!(file_id.chars().all(|value| value.is_ascii_hexdigit()));
    assert_eq!(*used_bytes.lock().await, 5);
    hashless_request.await.expect("hashless request");
    digest_request.await.expect("digest request");
    std::fs::remove_dir_all(directory).expect("remove cache");
}
