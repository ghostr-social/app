use crate::native_cache::NativeVideoCache;
use crate::native_cache_transfer::transfer;
use anyhow::{bail, Result};
use reqwest::RequestBuilder;
use std::path::Path;
use tokio::fs::File;

pub struct FetchedVideo {
    pub bytes: u64,
    pub content_length: Option<u64>,
    pub sha256: String,
}

pub async fn fetch(
    cache: &NativeVideoCache,
    request: RequestBuilder,
    path: &Path,
) -> Result<FetchedVideo> {
    let mut response = request.send().await?.error_for_status()?;
    if response.status() != reqwest::StatusCode::OK {
        bail!("native video response must be 200 OK");
    }
    let content_length = response.content_length();
    if let Some(bytes) = content_length {
        cache.ensure_capacity(bytes).await?;
    }
    let mut file = File::create(path).await?;
    let transferred = transfer(&mut response, &mut file, cache).await?;
    if transferred.bytes == 0 {
        bail!("native video response body is empty");
    }
    Ok(FetchedVideo {
        bytes: transferred.bytes,
        content_length,
        sha256: transferred.sha256,
    })
}
