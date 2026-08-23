//! Authoritative registry of progressive videos exposed by the cache.

use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::VideoMeta;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::sync::Notify;

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
    changed: Arc<Notify>,
}

#[derive(Debug, Default, Eq, PartialEq)]
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
        if entries.by_id.contains_key(&id) {
            return;
        }
        entries.order.push(id.clone());
        entries.by_id.insert(id, None);
        drop(entries);
        self.changed.notify_waiters();
    }

    pub fn replace(&self, videos: impl IntoIterator<Item = CacheVideo>) {
        let mut next = CacheEntries::default();
        for video in videos {
            remember(&mut next, video);
        }
        let mut entries = self.write();
        if *entries == next {
            return;
        }
        *entries = next;
        drop(entries);
        self.changed.notify_waiters();
    }

    pub fn contains(&self, id: &str) -> bool {
        self.read().by_id.contains_key(id)
    }

    pub fn matches_binding(&self, id: &str, binding: &RepresentationBinding) -> bool {
        self.video_for_binding(id, binding).is_some()
    }

    pub fn video_for_binding(
        &self,
        id: &str,
        binding: &RepresentationBinding,
    ) -> Option<CacheVideo> {
        if binding.post().as_str() != id {
            return None;
        }
        self.read()
            .by_id
            .get(id)
            .and_then(Option::as_ref)
            .filter(|video| binding.matches_or_derives_from(&video.meta))
            .cloned()
    }

    pub fn allows_binding(&self, id: &str, binding: &RepresentationBinding) -> bool {
        if binding.post().as_str() != id {
            return false;
        }
        match self.read().by_id.get(id) {
            Some(None) => true,
            Some(Some(video)) => binding.matches_or_derives_from(&video.meta),
            None => false,
        }
    }

    pub fn notifier(&self) -> Arc<Notify> {
        self.changed.clone()
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
