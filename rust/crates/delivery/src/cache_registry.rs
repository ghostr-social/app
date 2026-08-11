//! Authoritative registry of progressive videos exposed by the cache.

use ghostr_engine::VideoMeta;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheStatus {
    Ready,
    Partial,
    Complete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheVideo {
    pub id: String,
    pub meta: VideoMeta,
    pub status: CacheStatus,
}

#[derive(Clone, Debug, Default)]
pub struct CacheRegistry {
    inner: Arc<RwLock<CacheEntries>>,
}

#[derive(Debug, Default)]
struct CacheEntries {
    order: Vec<String>,
    by_id: HashMap<String, Option<CacheVideo>>,
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compatibility admission for a playback URL registered before
    /// delivery has published its metadata.
    pub fn insert(&self, id: impl Into<String>) {
        let id = id.into();
        let mut entries = self.write();
        if !entries.by_id.contains_key(&id) {
            entries.order.push(id.clone());
        }
        entries.by_id.insert(id, None);
    }

    pub fn replace(&self, videos: impl IntoIterator<Item = CacheVideo>) {
        let mut entries = self.write();
        entries.order.clear();
        entries.by_id.clear();
        for video in videos {
            remember(&mut entries, video);
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.read().by_id.contains_key(id)
    }

    pub fn videos(&self) -> Vec<CacheVideo> {
        let entries = self.read();
        entries
            .order
            .iter()
            .filter_map(|id| entries.by_id.get(id).cloned().flatten())
            .collect()
    }

    fn read(&self) -> RwLockReadGuard<'_, CacheEntries> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, CacheEntries> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn remember(entries: &mut CacheEntries, video: CacheVideo) {
    if !entries.by_id.contains_key(&video.id) {
        entries.order.push(video.id.clone());
    }
    entries.by_id.insert(video.id.clone(), Some(video));
}
