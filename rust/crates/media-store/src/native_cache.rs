use crate::native_blob_store::NativeBlobStore;
use crate::native_cache_capacity::{capacity_exhausted, NativeCacheCapacity};
use crate::native_cache_digest::verify_digest;
pub use crate::native_cache_directory::prepare_native_cache_directory;
use crate::native_cache_directory::{completed_path, install};
use crate::native_cache_fetch::{fetch, FetchedVideo};
use crate::native_cache_transfer::reserved_bytes;
use crate::native_partial_store::NativePartialStore;
use anyhow::Result;
use ghostr_media_model::native_models::NativeVideoCacheKey;
use ghostr_net::native_cache_failure::permanent;
use ghostr_net::outbound_media_client::MediaHttpClient;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct CachedVideo {
    pub path: PathBuf,
    pub bytes: u64,
    pub content_length: Option<u64>,
}

pub struct NativeVideoCache {
    blobs: NativeBlobStore,
    capacity: NativeCacheCapacity,
    directory: PathBuf,
    max_bytes: u64,
    partials: NativePartialStore,
    used_bytes: Arc<Mutex<u64>>,
}

impl NativeVideoCache {
    pub fn new(directory: PathBuf, max_bytes: u64, used_bytes: Arc<Mutex<u64>>) -> Self {
        Self::with_eviction_grace(directory, max_bytes, used_bytes, Duration::ZERO)
    }

    pub fn with_eviction_grace(
        directory: PathBuf,
        max_bytes: u64,
        used_bytes: Arc<Mutex<u64>>,
        eviction_grace: Duration,
    ) -> Self {
        Self {
            blobs: NativeBlobStore::new(used_bytes.clone(), eviction_grace),
            capacity: NativeCacheCapacity::default(),
            directory,
            max_bytes,
            partials: NativePartialStore::new(used_bytes.clone()),
            used_bytes,
        }
    }

    pub async fn download(
        &self,
        client: &MediaHttpClient,
        key: &NativeVideoCacheKey,
        url: &str,
        expected_digest: Option<&str>,
    ) -> Result<CachedVideo> {
        let request = client.get(url)?;
        self.download_request(key, expected_digest, request).await
    }

    async fn download_request(
        &self,
        key: &NativeVideoCacheKey,
        expected_digest: Option<&str>,
        request: reqwest::RequestBuilder,
    ) -> Result<CachedVideo> {
        let completed = completed_path(&self.directory, key)?;
        if let Some(cached) = self.blobs.find(key).await? {
            self.capacity.forget(key).await;
            return Ok(cached);
        }
        self.fetch_and_install(key, expected_digest, request, completed)
            .await
    }

    async fn fetch_and_install(
        &self,
        key: &NativeVideoCacheKey,
        expected_digest: Option<&str>,
        request: reqwest::RequestBuilder,
        completed: PathBuf,
    ) -> Result<CachedVideo> {
        let partial = completed.with_extension("partial");
        self.partials.reclaim_path(&partial).await?;
        let fetched = self.fetch_for_key(key, request, &partial).await?;
        verify_digest(&self.partials, &partial, expected_digest, &fetched).await?;
        install(&self.partials, &partial, &completed, fetched.bytes).await?;
        let cached = CachedVideo {
            path: completed,
            bytes: fetched.bytes,
            content_length: fetched.content_length,
        };
        self.blobs.remember(key.clone(), cached.clone()).await;
        Ok(cached)
    }

    async fn fetch_partial(
        &self,
        request: reqwest::RequestBuilder,
        partial: &Path,
    ) -> Result<FetchedVideo> {
        match fetch(self, request, partial).await {
            Ok(result) => Ok(result),
            Err(error) => {
                let bytes = reserved_bytes(&error);
                Err(self.partials.cleanup_error(partial, bytes, error).await)
            }
        }
    }

    async fn fetch_for_key(
        &self,
        key: &NativeVideoCacheKey,
        request: reqwest::RequestBuilder,
        partial: &Path,
    ) -> Result<FetchedVideo> {
        let result = self.fetch_after_check(request, partial).await;
        match &result {
            Ok(_) => self.capacity.forget(key).await,
            Err(error) => {
                let used = *self.used_bytes.lock().await;
                self.capacity.remember(key, error, used).await;
            }
        }
        result
    }

    async fn fetch_after_check(
        &self,
        request: reqwest::RequestBuilder,
        partial: &Path,
    ) -> Result<FetchedVideo> {
        self.ensure_capacity(1).await?;
        self.fetch_partial(request, partial).await
    }

    pub async fn retain(
        &self,
        active: &HashSet<NativeVideoCacheKey>,
    ) -> Result<HashSet<NativeVideoCacheKey>> {
        self.partials.reclaim_all().await;
        self.capacity.retain(active).await;
        self.blobs.retain(active).await
    }

    pub(crate) async fn reserve(&self, bytes: u64) -> Result<()> {
        let mut used = self.used_bytes.lock().await;
        let next = used
            .checked_add(bytes)
            .ok_or_else(|| capacity_exhausted(*used, bytes, self.max_bytes))?;
        if next > self.max_bytes {
            return Err(capacity_exhausted(*used, bytes, self.max_bytes));
        }
        *used = next;
        Ok(())
    }

    pub(crate) fn ensure_object_fits(&self, bytes: u64) -> Result<()> {
        (bytes <= self.max_bytes)
            .then_some(())
            .ok_or_else(|| permanent("native video exceeds cache capacity"))
    }

    pub(crate) async fn ensure_capacity(&self, bytes: u64) -> Result<()> {
        self.ensure_object_fits(bytes)?;
        let used = *self.used_bytes.lock().await;
        if used
            .checked_add(bytes)
            .is_none_or(|next| next > self.max_bytes)
        {
            return Err(capacity_exhausted(used, bytes, self.max_bytes));
        }
        Ok(())
    }
}
