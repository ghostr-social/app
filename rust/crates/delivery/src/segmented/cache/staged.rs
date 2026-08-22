use super::objects::insert;
use super::{CachedHlsObject, SegmentedCache, SegmentedPhase, SegmentedSnapshot};
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::PostId;

impl SegmentedCache {
    pub(crate) fn mark_stage_preparing(
        &self,
        post: &PostId,
        generation: u64,
        eta_ms: u64,
        maximum_bytes: u64,
    ) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get(post) else {
            return false;
        };
        if record.generation != generation || record.snapshot.phase == SegmentedPhase::Ready {
            return false;
        }
        let protected = record.protected;
        if !fits(&state, post, maximum_bytes) && protected {
            super::objects::reclaim_unprotected_ready(&mut state);
        }
        if !fits(&state, post, maximum_bytes) {
            return false;
        }
        let record = state
            .focus
            .get_mut(post)
            .expect("validated HLS cache focus");
        record.snapshot.phase = SegmentedPhase::Preparing;
        record.snapshot.eta_ms = Some(eta_ms);
        record.snapshot.detail = None;
        record.reserved_bytes = maximum_bytes;
        drop(state);
        self.changed.notify_waiters();
        true
    }

    pub(in crate::segmented) fn store_stage_object(
        &self,
        post: &PostId,
        generation: u64,
        object: PreparedObject,
    ) -> Option<u64> {
        let mut state = self.lock();
        let record = state.focus.get_mut(post)?;
        if record.generation != generation || record.snapshot.phase == SegmentedPhase::Ready {
            return None;
        }
        if object.body.len() as u64 > record.reserved_bytes {
            return None;
        }
        record.reserved_bytes = 0;
        match record
            .staged
            .iter()
            .position(|known| known.request_url == object.request_url)
        {
            Some(index) => record.staged[index] = object,
            None => record.staged.push(object),
        }
        record.snapshot.bytes_present = record
            .staged
            .iter()
            .map(|known| known.body.len() as u64)
            .sum();
        record.snapshot.phase = SegmentedPhase::Queued;
        record.snapshot.eta_ms = None;
        record.snapshot.detail = None;
        let total = record.snapshot.bytes_present;
        drop(state);
        self.changed.notify_waiters();
        Some(total)
    }

    pub(crate) fn mark_stage_ready(&self, post: &PostId, generation: u64) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation != generation {
            return false;
        }
        let staged = std::mem::take(&mut record.staged);
        record.reserved_bytes = 0;
        let keys = staged
            .iter()
            .map(|object| object.request_url.clone())
            .collect::<Vec<_>>();
        for object in staged {
            let cached = CachedHlsObject::new(object.body, object.final_url, object.content_type);
            insert(&mut state, object.request_url, cached);
        }
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        record.objects = keys;
        record.snapshot.phase = SegmentedPhase::Ready;
        record.snapshot.eta_ms = Some(0);
        record.snapshot.detail = None;
        drop(state);
        self.changed.notify_waiters();
        true
    }

    pub(crate) fn mark_stage_failed(&self, post: &PostId, generation: u64, detail: String) -> bool {
        self.mark_terminal(post, generation, SegmentedPhase::Failed, Some(detail))
    }

    pub(crate) fn reset_stage_retry(&self, post: &PostId, generation: u64) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation != generation {
            return false;
        }
        record.staged.clear();
        record.reserved_bytes = 0;
        record.snapshot = SegmentedSnapshot::default();
        drop(state);
        self.changed.notify_waiters();
        true
    }

    fn mark_terminal(
        &self,
        post: &PostId,
        generation: u64,
        phase: SegmentedPhase,
        detail: Option<String>,
    ) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation != generation {
            return false;
        }
        record.snapshot.phase = phase;
        if phase == SegmentedPhase::Failed {
            record.staged.clear();
            record.reserved_bytes = 0;
            record.snapshot.bytes_present = 0;
        }
        record.snapshot.eta_ms = (phase == SegmentedPhase::Ready).then_some(0);
        record.snapshot.detail = detail;
        drop(state);
        self.changed.notify_waiters();
        true
    }
}

fn fits(state: &super::CacheState, post: &PostId, maximum_bytes: u64) -> bool {
    let current = state
        .focus
        .get(post)
        .map_or(0, |record| record.reserved_bytes);
    let used_without_current = physical_used(state).saturating_sub(current);
    maximum_bytes <= (super::MAX_CACHE_BYTES as u64).saturating_sub(used_without_current)
}

pub(super) fn physical_used(state: &super::CacheState) -> u64 {
    let staged = state
        .focus
        .values()
        .flat_map(|record| &record.staged)
        .map(|object| object.body.len() as u64)
        .sum::<u64>();
    let reserved = state
        .focus
        .values()
        .map(|record| record.reserved_bytes)
        .sum::<u64>();
    (state.bytes as u64)
        .saturating_add(staged)
        .saturating_add(reserved)
}
