use crate::video::partial_range_disk as disk;
use crate::video::partial_range_disk::Entry;
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// Sparse on-disk store of partially downloaded videos: per key one data file
/// written at byte offsets plus a persisted manifest of the present ranges.
pub struct PartialRangeStore {
    root: PathBuf,
    entries: Mutex<HashMap<String, Entry>>,
    used_bytes: Arc<Mutex<u64>>,
    changed: Arc<Notify>,
}

type Entries = HashMap<String, Entry>;

impl PartialRangeStore {
    pub fn new(root: PathBuf, used_bytes: Arc<Mutex<u64>>) -> Self {
        Self {
            root,
            entries: Mutex::new(HashMap::new()),
            used_bytes,
            changed: Arc::new(Notify::new()),
        }
    }

    /// Woken (`notify_waiters`) after every range write and every total
    /// length declaration; readers register before re-checking the store.
    pub fn change_notifier(&self) -> Arc<Notify> {
        self.changed.clone()
    }

    pub async fn total_len(&self, key: &str) -> Result<Option<u64>> {
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.manifest.total_len())
    }

    pub async fn write_range(&self, key: &str, offset: u64, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let end = offset
            .checked_add(bytes.len() as u64)
            .context("partial range end overflows")?;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completed {
            bail!("cannot write into a finalized video");
        }
        let mut updated = entry.manifest.clone();
        updated.insert(offset..end)?;
        disk::write_at(&self.partial_path(key), offset, bytes).await?;
        disk::save_manifest(&self.manifest_path(key), &updated).await?;
        let added = updated.covered_bytes() - entry.manifest.covered_bytes();
        entry.manifest = updated;
        entry.accounted += added;
        self.credit(added).await;
        self.changed.notify_waiters();
        Ok(())
    }

    pub async fn present_ranges(&self, key: &str) -> Result<Vec<Range<u64>>> {
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.manifest.ranges())
    }

    pub async fn missing_within(&self, key: &str, span: Range<u64>) -> Result<Vec<Range<u64>>> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.manifest.missing_within(&span))
    }

    pub async fn read_range(&self, key: &str, span: Range<u64>) -> Result<Option<Vec<u8>>> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if !entry.manifest.contains(&span) {
            return Ok(None);
        }
        let path = if entry.completed {
            self.completed_path(key)
        } else {
            self.partial_path(key)
        };
        disk::read_span(&path, &span).await.map(Some)
    }

    pub async fn set_total_len(&self, key: &str, len: u64) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completed {
            bail!("cannot resize a finalized video");
        }
        entry.manifest.set_total_len(len)?;
        disk::save_manifest(&self.manifest_path(key), &entry.manifest).await?;
        self.changed.notify_waiters();
        Ok(())
    }

    pub async fn is_complete(&self, key: &str) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.completed || entry.manifest.is_complete())
    }

    pub async fn finalize(&self, key: &str, expected_sha256_hex: &str) -> Result<PathBuf> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completed {
            return Ok(self.completed_path(key));
        }
        if !entry.manifest.is_complete() {
            bail!("cannot finalize a video with missing ranges");
        }
        let digest = disk::sha256_file(&self.partial_path(key)).await?;
        if digest.eq_ignore_ascii_case(expected_sha256_hex) {
            return self.promote(&mut entries, key).await;
        }
        self.discard(&mut entries, key).await?;
        bail!("partial video digest does not match the expected digest")
    }

    pub async fn remove(&self, key: &str) -> Result<()> {
        let mut entries = self.entries.lock().await;
        self.entry(&mut entries, key).await?;
        self.discard(&mut entries, key).await
    }

    pub fn completed_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.video"))
    }

    fn partial_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.part"))
    }

    fn manifest_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.ranges.json"))
    }

    async fn promote(&self, entries: &mut Entries, key: &str) -> Result<PathBuf> {
        let completed = self.completed_path(key);
        tokio::fs::rename(&self.partial_path(key), &completed)
            .await
            .context("promote complete partial video")?;
        disk::remove_if_present(&self.manifest_path(key)).await?;
        let entry = entries.get_mut(key).context("promoted entry present")?;
        entry.completed = true;
        Ok(completed)
    }

    async fn discard(&self, entries: &mut Entries, key: &str) -> Result<()> {
        disk::remove_if_present(&self.partial_path(key)).await?;
        disk::remove_if_present(&self.manifest_path(key)).await?;
        disk::remove_if_present(&self.completed_path(key)).await?;
        if let Some(entry) = entries.remove(key) {
            self.release(entry.accounted).await;
        }
        Ok(())
    }

    async fn entry<'a>(&self, entries: &'a mut Entries, key: &str) -> Result<&'a mut Entry> {
        validate_key(key)?;
        if !entries.contains_key(key) {
            let loaded =
                disk::load_entry(&self.completed_path(key), &self.manifest_path(key)).await?;
            self.credit(loaded.accounted).await;
            entries.insert(key.to_owned(), loaded);
        }
        entries.get_mut(key).context("partial entry present")
    }

    async fn credit(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_add(bytes);
    }

    async fn release(&self, bytes: u64) {
        let mut used = self.used_bytes.lock().await;
        *used = used.saturating_sub(bytes);
    }
}

fn validate_key(key: &str) -> Result<()> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_');
    if !key.is_empty() && key.chars().all(allowed) {
        return Ok(());
    }
    bail!("partial store keys must be alphanumeric with dashes or underscores")
}
