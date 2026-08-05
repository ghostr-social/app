use super::fixtures::{temp_directory, trusted_media_client};
use super::http::spawn_raw_server;
use rust_lib_ghostr::video::native_cache::{
    prepare_native_cache_directory, CachedVideo, NativeVideoCache,
};
use rust_lib_ghostr::video::native_models::NativeVideoCacheKey;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const VIDEO_DIGEST: &str = "0cab1c9617404faf2b24e221e189ca5945813e14d3f766345b09ca13bbe28ffc";
pub const VIDEO_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";

pub struct NativeCacheHarness {
    pub cache: NativeVideoCache,
    pub directory: PathBuf,
    pub used_bytes: Arc<Mutex<u64>>,
}

impl NativeCacheHarness {
    pub fn new(prefix: &str) -> Self {
        let directory = temp_directory(prefix);
        prepare_native_cache_directory(&directory).expect("prepare cache");
        let used_bytes = Arc::new(Mutex::new(0));
        Self {
            cache: NativeVideoCache::new(directory.clone(), 20, used_bytes.clone()),
            directory,
            used_bytes,
        }
    }

    pub async fn download(
        &self,
        key: &NativeVideoCacheKey,
        response: &'static [u8],
        expected: Option<&str>,
    ) -> CachedVideo {
        let (url, request) = spawn_raw_server(response).await;
        let cached = self
            .cache
            .download(&trusted_media_client(), key, &url, expected)
            .await
            .expect("cached video");
        request.await.expect("upstream request");
        cached
    }
}

pub fn advertised_key() -> NativeVideoCacheKey {
    NativeVideoCacheKey::AdvertisedDigest(VIDEO_DIGEST.to_owned())
}
