use crate::native_blob_integrity::{remove_if_present, validate_blob, NativeBlobSnapshot};
use crate::native_cache::CachedVideo;
use anyhow::Result;
use ghostr_media_model::native_models::NativeVideoCacheKey;
use log::warn;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::Instant;

struct NativeBlobEntry {
    modified: Option<SystemTime>,
    orphaned_at: Option<Instant>,
    video: CachedVideo,
}

pub struct NativeBlobStore {
    entries: Mutex<HashMap<NativeVideoCacheKey, NativeBlobEntry>>,
    eviction_grace: Duration,
    used_bytes: Arc<Mutex<u64>>,
}

impl NativeBlobStore {
    pub(crate) fn new(used_bytes: Arc<Mutex<u64>>, eviction_grace: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            eviction_grace,
            used_bytes,
        }
    }

    pub(crate) async fn find(&self, key: &NativeVideoCacheKey) -> Result<Option<CachedVideo>> {
        let Some(snapshot) = self.entry(key).await else {
            return Ok(None);
        };
        let validation = validate_blob(&snapshot).await?;
        if !validation.valid {
            self.remove(key, &snapshot.video).await?;
            return Ok(None);
        }
        self.mark_valid(key, validation.modified).await;
        Ok(Some(snapshot.video))
    }

    pub(crate) async fn remember(&self, key: NativeVideoCacheKey, video: CachedVideo) {
        let entry = NativeBlobEntry {
            modified: modified_time(&video).await,
            orphaned_at: None,
            video,
        };
        self.entries.lock().await.insert(key, entry);
    }

    pub(crate) async fn retain(
        &self,
        active: &HashSet<NativeVideoCacheKey>,
    ) -> Result<HashSet<NativeVideoCacheKey>> {
        let invalid = self.invalid_active_entries(active).await?;
        for (id, video) in &invalid {
            if let Err(error) = self.remove(id, video).await {
                warn!("Native cache could not remove an invalid blob: {error:#}");
            }
        }
        let stale = self.stale_entries(active).await;
        for (id, video) in stale {
            if let Err(error) = self.remove(&id, &video).await {
                warn!("Native cache could not remove stale blob: {error:#}");
            }
        }
        Ok(invalid.into_iter().map(|(id, _)| id).collect())
    }

    async fn entry(&self, key: &NativeVideoCacheKey) -> Option<NativeBlobSnapshot> {
        self.entries
            .lock()
            .await
            .get(key)
            .map(|entry| NativeBlobSnapshot {
                key: key.clone(),
                modified: entry.modified,
                video: entry.video.clone(),
            })
    }

    async fn mark_valid(&self, key: &NativeVideoCacheKey, modified: Option<SystemTime>) {
        if let Some(entry) = self.entries.lock().await.get_mut(key) {
            entry.modified = modified;
            entry.orphaned_at = None;
        }
    }

    async fn invalid_active_entries(
        &self,
        active: &HashSet<NativeVideoCacheKey>,
    ) -> Result<Vec<(NativeVideoCacheKey, CachedVideo)>> {
        let candidates = self
            .entries
            .lock()
            .await
            .iter()
            .filter(|(key, _)| active.contains(*key))
            .map(|(key, entry)| NativeBlobSnapshot {
                key: key.clone(),
                modified: entry.modified,
                video: entry.video.clone(),
            })
            .collect::<Vec<_>>();
        let mut invalid = Vec::new();
        for candidate in candidates {
            let validation = validate_blob(&candidate).await?;
            if validation.valid {
                self.mark_valid(&candidate.key, validation.modified).await;
            } else {
                invalid.push((candidate.key, candidate.video));
            }
        }
        Ok(invalid)
    }

    async fn stale_entries(
        &self,
        active: &HashSet<NativeVideoCacheKey>,
    ) -> Vec<(NativeVideoCacheKey, CachedVideo)> {
        let now = Instant::now();
        let mut entries = self.entries.lock().await;
        entries
            .iter_mut()
            .filter_map(|(id, entry)| self.stale_entry(id, entry, active, now))
            .collect()
    }

    fn stale_entry(
        &self,
        key: &NativeVideoCacheKey,
        entry: &mut NativeBlobEntry,
        active: &HashSet<NativeVideoCacheKey>,
        now: Instant,
    ) -> Option<(NativeVideoCacheKey, CachedVideo)> {
        if active.contains(key) {
            entry.orphaned_at = None;
            return None;
        }
        let orphaned_at = *entry.orphaned_at.get_or_insert(now);
        (now.duration_since(orphaned_at) >= self.eviction_grace)
            .then(|| (key.clone(), entry.video.clone()))
    }

    async fn remove(&self, key: &NativeVideoCacheKey, video: &CachedVideo) -> Result<()> {
        remove_if_present(&video.path).await?;
        let removed = self.entries.lock().await.remove(key);
        if removed.is_some() {
            let mut used = self.used_bytes.lock().await;
            *used = used.saturating_sub(video.bytes);
        }
        Ok(())
    }
}

async fn modified_time(video: &CachedVideo) -> Option<SystemTime> {
    tokio::fs::symlink_metadata(&video.path)
        .await
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}
