use super::PreparedHls;
use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;

mod generation;
mod objects;
#[cfg(test)]
mod tests;
pub use generation::{CachedHlsGeneration, CachedHlsObject};
use objects::{commit, failed};

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
    snapshot: SegmentedSnapshot,
    objects: Vec<String>,
}

impl SegmentedCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn replace_focus(&self, generation: u64, items: Vec<(PostId, Vec<String>)>) {
        let mut state = self.lock();
        let mut next = HashMap::new();
        for (post, sources) in items {
            let (snapshot, objects) = reusable_state(state.focus.get(&post), &sources);
            next.insert(
                post,
                FocusRecord {
                    generation,
                    sources,
                    snapshot,
                    objects,
                },
            );
        }
        state.focus = next;
        drop(state);
        self.changed.notify_waiters();
    }

    pub(crate) fn mark_preparing(&self, post: &PostId, generation: u64, eta_ms: u64) -> bool {
        self.update(
            post,
            generation,
            SegmentedSnapshot {
                phase: SegmentedPhase::Preparing,
                bytes_present: 0,
                eta_ms: Some(eta_ms),
                detail: None,
            },
        )
    }

    pub(crate) fn complete(
        &self,
        post: &PostId,
        generation: u64,
        result: anyhow::Result<PreparedHls>,
    ) {
        let mut state = self.lock();
        let current = state
            .focus
            .get(post)
            .is_some_and(|record| record.generation == generation);
        if !current {
            return;
        }
        let committed = match result {
            Ok(prepared) => commit(&mut state, prepared),
            Err(error) => failed(error.to_string()),
        };
        if let Some(record) = state.focus.get_mut(post) {
            record.snapshot = committed.snapshot;
            record.objects = committed.objects;
        }
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

    fn update(&self, post: &PostId, generation: u64, snapshot: SegmentedSnapshot) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation != generation || record.snapshot.phase == SegmentedPhase::Ready {
            return false;
        }
        record.snapshot = snapshot;
        drop(state);
        self.changed.notify_waiters();
        true
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
