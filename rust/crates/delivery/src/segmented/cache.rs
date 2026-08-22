use super::prepare::PreparedObject;
use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;

mod capacity;
mod generation;
mod objects;
mod staged;
#[cfg(test)]
mod tests;
pub use generation::{CachedHlsGeneration, CachedHlsObject};

const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentedPhase {
    Queued,
    Preparing,
    Ready,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentedSnapshot {
    pub phase: SegmentedPhase,
    pub bytes_present: u64,
    pub eta_ms: Option<u64>,
    pub detail: Option<String>,
}

impl Default for SegmentedSnapshot {
    fn default() -> Self {
        Self {
            phase: SegmentedPhase::Queued,
            bytes_present: 0,
            eta_ms: None,
            detail: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct SegmentedCache {
    state: Arc<Mutex<CacheState>>,
    changed: Arc<Notify>,
}

#[derive(Default)]
struct CacheState {
    focus: HashMap<PostId, FocusRecord>,
    objects: HashMap<String, CachedHlsObject>,
    aliases: HashMap<String, String>,
    order: VecDeque<String>,
    bytes: usize,
}

struct FocusRecord {
    generation: u64,
    sources: Vec<String>,
    protected: bool,
    snapshot: SegmentedSnapshot,
    objects: Vec<String>,
    staged: Vec<PreparedObject>,
    reserved_bytes: u64,
}

impl SegmentedCache {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn replace_focus(&self, generation: u64, items: Vec<(PostId, Vec<String>)>) {
        let protected = items.iter().map(|(post, _)| post.clone()).collect();
        self.replace_focus_window(generation, items, &protected);
    }

    pub(crate) fn replace_focus_window(
        &self,
        generation: u64,
        items: Vec<(PostId, Vec<String>)>,
        protected: &std::collections::HashSet<PostId>,
    ) {
        let mut state = self.lock();
        let mut next = HashMap::new();
        for (post, sources) in items {
            let (snapshot, objects) = reusable_state(state.focus.get(&post), &sources);
            let is_protected = protected.contains(&post);
            next.insert(
                post,
                FocusRecord {
                    generation,
                    sources,
                    protected: is_protected,
                    snapshot,
                    objects,
                    staged: Vec::new(),
                    reserved_bytes: 0,
                },
            );
        }
        state.focus = next;
        objects::retain_referenced(&mut state);
        drop(state);
        self.changed.notify_waiters();
    }

    pub fn snapshot(&self, post: &str) -> SegmentedSnapshot {
        self.lock()
            .focus
            .get(&PostId::new(post))
            .map(|record| record.snapshot.clone())
            .unwrap_or_default()
    }

    pub fn object(&self, url: &str) -> Option<CachedHlsObject> {
        let state = self.lock();
        state
            .objects
            .get(url)
            .or_else(|| {
                state
                    .aliases
                    .get(url)
                    .and_then(|key| state.objects.get(key))
            })
            .cloned()
    }

    pub fn notifier(&self) -> Arc<Notify> {
        self.changed.clone()
    }

    pub fn clear(&self) {
        *self.lock() = CacheState::default();
        self.changed.notify_waiters();
    }

    fn lock(&self) -> MutexGuard<'_, CacheState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

fn reusable_state(
    previous: Option<&FocusRecord>,
    sources: &[String],
) -> (SegmentedSnapshot, Vec<String>) {
    match previous {
        Some(record)
            if record.sources == sources && record.snapshot.phase == SegmentedPhase::Ready =>
        {
            (record.snapshot.clone(), record.objects.clone())
        }
        _ => (SegmentedSnapshot::default(), Vec::new()),
    }
}
