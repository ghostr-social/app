use anyhow::{bail, Context, Result};
use reqwest::{Client, Response};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub struct CachedVideo {
    pub path: PathBuf,
    pub bytes: u64,
    pub content_length: Option<u64>,
}

pub struct NativeVideoCache {
    directory: PathBuf,
    max_bytes: u64,
    used_bytes: Arc<Mutex<u64>>,
}

impl NativeVideoCache {
    pub fn new(directory: PathBuf, max_bytes: u64, used_bytes: Arc<Mutex<u64>>) -> Self {
        Self {
            directory,
            max_bytes,
            used_bytes,
        }
    }

    pub async fn download(&self, client: &Client, id: &str, url: &str) -> Result<CachedVideo> {
        let completed = self.completed_path(id)?;
        self.ensure_capacity(1).await?;
        let partial = completed.with_extension("partial");
        let transferred = self.fetch(client, url, &partial).await;
        let (bytes, content_length) = match transferred {
            Ok(result) => result,
            Err(error) => return Err(cleanup_error(&partial, error).await),
        };
        self.install(&partial, &completed, bytes).await?;
        Ok(CachedVideo {
            path: completed,
            bytes,
            content_length,
        })
    }

    async fn fetch(&self, client: &Client, url: &str, path: &Path) -> Result<(u64, Option<u64>)> {
        let mut response = client.get(url).send().await?.error_for_status()?;
        let content_length = response.content_length();
        if let Some(bytes) = content_length {
            self.ensure_capacity(bytes).await?;
        }
        let mut file = File::create(path).await?;
        let bytes = transfer(&mut response, &mut file, self).await?;
        Ok((bytes, content_length))
    }

    async fn install(&self, partial: &Path, completed: &Path, bytes: u64) -> Result<()> {
        if let Err(error) = tokio::fs::rename(partial, completed).await {
            self.release(bytes).await;
            return Err(cleanup_error(partial, error.into()).await);
        }
        Ok(())
    }

    async fn reserve(&self, bytes: u64) -> Result<()> {
        let mut used = self.used_bytes.lock().await;
        let next = used.checked_add(bytes).context("native cache overflow")?;
        if next > self.max_bytes {
            bail!("native video cache budget exhausted");
        }
        *used = next;
        Ok(())
    }

    async fn ensure_capacity(&self, bytes: u64) -> Result<()> {
        let used = *self.used_bytes.lock().await;
        if used
            .checked_add(bytes)
            .is_none_or(|next| next > self.max_bytes)
        {
            bail!("native video cache budget exhausted");
        }
        Ok(())
    }

    async fn release(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_sub(bytes);
    }

    fn completed_path(&self, id: &str) -> Result<PathBuf> {
        if id.len() != 64 || !id.chars().all(|value| value.is_ascii_hexdigit()) {
            bail!("native video cache identifier is invalid");
        }
        Ok(self.directory.join(format!("{id}.mp4")))
    }
}

async fn transfer(
    response: &mut Response,
    file: &mut File,
    cache: &NativeVideoCache,
) -> Result<u64> {
    let mut transferred = 0;
    while let Some(chunk) = read_chunk(response, cache, transferred).await? {
        let bytes = chunk.len() as u64;
        reserve_chunk(cache, bytes, transferred).await?;
        if let Err(error) = file.write_all(&chunk).await {
            cache.release(transferred + bytes).await;
            return Err(error.into());
        }
        transferred += bytes;
    }
    Ok(transferred)
}

async fn read_chunk(
    response: &mut Response,
    cache: &NativeVideoCache,
    transferred: u64,
) -> Result<Option<bytes::Bytes>> {
    match response.chunk().await {
        Ok(chunk) => Ok(chunk),
        Err(error) => {
            cache.release(transferred).await;
            Err(error.into())
        }
    }
}

async fn reserve_chunk(cache: &NativeVideoCache, bytes: u64, transferred: u64) -> Result<()> {
    if let Err(error) = cache.reserve(bytes).await {
        cache.release(transferred).await;
        return Err(error);
    }
    Ok(())
}

async fn cleanup_error(path: &Path, error: anyhow::Error) -> anyhow::Error {
    match remove_if_present(path).await {
        Ok(()) => error,
        Err(cleanup) => error.context(format!("partial-file cleanup also failed: {cleanup}")),
    }
}

async fn remove_if_present(path: &Path) -> Result<()> {
    if tokio::fs::try_exists(path)
        .await
        .context("inspect native partial file")?
    {
        tokio::fs::remove_file(path)
            .await
            .context("remove native partial file")?;
    }
    Ok(())
}

pub fn prepare_native_cache_directory(directory: &Path) -> Result<()> {
    if directory.exists() {
        fs::remove_dir_all(directory).context("clear native video cache")?;
    }
    fs::create_dir_all(directory).context("create native video cache")?;
    Ok(())
}
