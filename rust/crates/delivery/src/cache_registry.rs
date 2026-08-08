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
    inner: Arc<RwLock<HashMap<String, Option<CacheVideo>>>>,
}

impl CacheRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compatibility admission for a playback URL registered before
    /// delivery has published its metadata.
    pub fn insert(&self, id: impl Into<String>) {
        self.write().insert(id.into(), None);
    }

    pub fn replace(&self, videos: impl IntoIterator<Item = CacheVideo>) {
        let mut guard = self.write();
        guard.clear();
        guard.extend(
            videos
                .into_iter()
                .map(|video| (video.id.clone(), Some(video))),
        );
    }

    pub fn contains(&self, id: &str) -> bool {
        self.read().contains_key(id)
    }

    pub fn videos(&self) -> Vec<CacheVideo> {
        let mut videos: Vec<_> = self.read().values().filter_map(Clone::clone).collect();
        videos.sort_by(|left, right| left.id.cmp(&right.id));
        videos
    }

    fn read(&self) -> RwLockReadGuard<'_, HashMap<String, Option<CacheVideo>>> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, HashMap<String, Option<CacheVideo>>> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
