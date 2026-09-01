//! API-side registry of the focused posts the event stream
//! watches, plus the current data-usage level for startability math.

use crate::engine::{DataUsageLevel, VideoMeta};
use flutter_rust_bridge::frb;
use ghostr_delivery::delivery_events::FocusGeneration;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::sync::Notify;

#[derive(Clone)]
#[frb(ignore)]
pub(crate) struct TrackedItems {
    inner: Arc<RwLock<Tracked>>,
    changed: Arc<Notify>,
}

struct Tracked {
    items: HashMap<String, VideoMeta>,
    level: DataUsageLevel,
    latest_focus_generation: Option<u64>,
}

impl TrackedItems {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Tracked {
                items: HashMap::new(),
                level: DataUsageLevel::Balanced,
                latest_focus_generation: None,
            })),
            changed: Arc::new(Notify::new()),
        }
    }

    /// Atomically replaces watched items only for a newer focus.
    pub(crate) fn replace_focus(
        &self,
        generation: FocusGeneration,
        entries: Vec<(String, VideoMeta)>,
    ) -> bool {
        let Some(generation) = generation.value() else {
            return false;
        };
        let mut tracked = self.write();
        if tracked
            .latest_focus_generation
            .is_some_and(|latest| generation <= latest)
        {
            return false;
        }
        tracked.latest_focus_generation = Some(generation);
        tracked.items = entries.into_iter().collect();
        drop(tracked);
        self.changed.notify_waiters();
        true
    }

    /// Adds one post (playback registration). It survives only until
    /// the next focus replacement, exactly like the servable registry.
    pub(crate) fn insert(&self, id: String, meta: VideoMeta) {
        self.write().items.insert(id, meta);
        self.changed.notify_waiters();
    }

    pub(crate) fn set_level(&self, level: DataUsageLevel) {
        self.write().level = level;
        self.changed.notify_waiters();
    }

    /// The newest focus generation the watcher tracks, if any.
    pub(crate) fn focus_generation(&self) -> Option<u64> {
        self.read().latest_focus_generation
    }

    pub(crate) fn level(&self) -> DataUsageLevel {
        self.read().level
    }

    pub(crate) fn meta(&self, id: &str) -> Option<VideoMeta> {
        self.read().items.get(id).cloned()
    }

    /// Entries in stable id order so event emission is deterministic.
    pub(crate) fn snapshot(&self) -> Vec<(String, VideoMeta)> {
        let mut entries: Vec<_> = self.read().items.clone().into_iter().collect();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries
    }

    /// Woken after every registry or level change.
    pub(crate) fn notifier(&self) -> Arc<Notify> {
        std::sync::Arc::clone(&self.changed)
    }

    fn read(&self) -> RwLockReadGuard<'_, Tracked> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, Tracked> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
