use anyhow::{Context, Error, Result};
use log::warn;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NativePartialStore {
    entries: Mutex<HashMap<PathBuf, u64>>,
    used_bytes: Arc<Mutex<u64>>,
}

impl NativePartialStore {
    pub fn new(used_bytes: Arc<Mutex<u64>>) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            used_bytes,
        }
    }

    pub async fn cleanup_error(&self, path: &Path, bytes: u64, error: Error) -> Error {
        match remove_if_present(path).await {
            Ok(()) => {
                self.release(bytes).await;
                error
            }
            Err(cleanup) => {
                self.track(path, bytes).await;
                error.context(format!("partial-file cleanup also failed: {cleanup}"))
            }
        }
    }

    pub async fn reclaim_all(&self) {
        let paths = self
            .entries
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        for path in paths {
            if let Err(error) = self.reclaim_path(&path).await {
                warn!("Native partial cache cleanup deferred for {path:?}: {error}");
            }
        }
    }

    pub async fn reclaim_path(&self, path: &Path) -> Result<()> {
        if !self.entries.lock().await.contains_key(path) {
            return Ok(());
        }
        remove_if_present(path).await?;
        if let Some(bytes) = self.entries.lock().await.remove(path) {
            self.release(bytes).await;
        }
        Ok(())
    }

    async fn track(&self, path: &Path, bytes: u64) {
        let physical = physical_bytes(path).await.unwrap_or(bytes);
        let mut entries = self.entries.lock().await;
        let previous = entries.insert(path.to_path_buf(), physical).unwrap_or(0);
        drop(entries);
        let mut used = self.used_bytes.lock().await;
        *used = used
            .saturating_sub(bytes)
            .saturating_sub(previous)
            .saturating_add(physical);
    }

    async fn release(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_sub(bytes);
    }
}

async fn physical_bytes(path: &Path) -> Option<u64> {
    let metadata = tokio::fs::symlink_metadata(path).await.ok()?;
    metadata.file_type().is_file().then_some(metadata.len())
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
