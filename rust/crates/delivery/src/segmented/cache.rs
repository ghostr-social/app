use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::{watch, Notify};

mod blocks;
mod capacity;

mod admission;
pub(crate) use admission::{StageAdmission, StageFence, StageLease, StageRequest};
mod authority;
pub use authority::{HlsPreparedAssetAuthority, SegmentedAssetRevision};
mod focus;
pub(crate) use focus::{PreservedFocus, SegmentedFocusItem};
mod freshness;
mod generation;
pub(in crate::segmented) use generation::{CachedHlsGenerationHasher, HlsCacheMetadata};
mod invalidation;
mod objects;
mod staged;
mod staged_object;
pub(in crate::segmented) use staged_object::AssemblySeed;
use staged_object::StagedObject;
#[cfg(test)]
mod tests;
pub use generation::{CachedHlsGeneration, CachedHlsObject};

const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct StageReservation {
    block_bytes: u64,
    assembly_bytes: u64,
}

impl StageReservation {
    pub(super) const fn block(block_bytes: u64) -> Self {
        Self {
            block_bytes,
            assembly_bytes: 0,
        }
    }

    pub(super) fn final_block(block_bytes: u64, total_bytes: u64) -> Option<Self> {
        if block_bytes == 0 || total_bytes < block_bytes {
            return None;
        }
        block_bytes.checked_add(total_bytes)?;
        Some(Self {
            block_bytes,
            assembly_bytes: total_bytes,
        })
    }

    fn total_bytes(self) -> Option<u64> {
        self.block_bytes.checked_add(self.assembly_bytes)
    }
}

impl From<u64> for StageReservation {
    fn from(block_bytes: u64) -> Self {
        Self::block(block_bytes)
    }
}

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
    pub authority: Option<HlsPreparedAssetAuthority>,
}

impl Default for SegmentedSnapshot {
    fn default() -> Self {
        Self {
            phase: SegmentedPhase::Queued,
            bytes_present: 0,
            eta_ms: None,
            detail: None,
            authority: None,
        }
    }
}

#[derive(Clone)]
pub struct SegmentedCache {
    state: Arc<Mutex<CacheState>>,
    changed: Arc<Notify>,
    invalidations: watch::Sender<u64>,
}

#[derive(Default)]
struct CacheState {
    focus: HashMap<PostId, FocusRecord>,
    objects: HashMap<String, CachedHlsObject>,
    aliases: HashMap<String, String>,
    canonical_aliases: HashMap<String, String>,
    order: VecDeque<String>,
    invalidated: Vec<(PostId, u64)>,
    inflight: HashMap<admission::InflightKey, admission::InflightStage>,
    bytes: usize,
    last_asset_revision: u64,
}

struct FocusRecord {
    generation: u64,
    representation_id: ghostr_engine::representation::RepresentationId,
    sources: Vec<String>,
    root_source: Option<String>,
    protected: bool,
    snapshot: SegmentedSnapshot,
    objects: Vec<String>,
    staged: Vec<StagedObject>,
    preparing: Option<StageFence>,
    reserved_bytes: u64,
    assembly_bytes: u64,
}

impl Default for SegmentedCache {
    fn default() -> Self {
        let (invalidations, _) = watch::channel(0);
        Self {
            state: Arc::default(),
            changed: Arc::default(),
            invalidations,
        }
    }
}

impl SegmentedCache {
    pub fn new() -> Self {
        Self::default()
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
        let key = objects::resolve_key(&state, url)?;
        state.objects.get(&key).cloned()
    }

    pub fn reusable_object(&self, url: &str) -> Option<CachedHlsObject> {
        self.object(url).filter(CachedHlsObject::is_reusable)
    }

    pub fn notifier(&self) -> Arc<Notify> {
        std::sync::Arc::clone(&self.changed)
    }

    pub(crate) fn invalidation_receiver(&self) -> watch::Receiver<u64> {
        self.invalidations.subscribe()
    }

    pub fn clear(&self) {
        let mut state = self.lock();
        let inflight = core::mem::take(&mut state.inflight);
        let last_asset_revision = state.last_asset_revision;
        *state = CacheState {
            inflight,
            last_asset_revision,
            ..CacheState::default()
        };
        drop(state);
        self.changed.notify_waiters();
    }

    fn lock(&self) -> MutexGuard<'_, CacheState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

#[cfg(test)]
#[path = "cache_axiom_test.rs"]
pub(crate) mod axiom_test_support;
