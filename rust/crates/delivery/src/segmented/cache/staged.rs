use super::objects::insert;

use super::{
    CachedHlsObject, HlsPreparedAssetAuthority, SegmentedAssetRevision, SegmentedCache,
    SegmentedPhase, SegmentedSnapshot,
};
use ghostr_engine::PostId;

impl SegmentedCache {
    pub(crate) fn mark_stage_ready(&self, post: &PostId, generation: u64) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation != generation
            || record.preparing.is_some()
            || record.assembly_bytes != 0
            || !record.staged.iter().all(|object| object.is_assembled())
        {
            return false;
        }
        let staged = core::mem::take(&mut record.staged);
        record.reserved_bytes = 0;
        let staged = staged
            .into_iter()
            .map(|object| object.into_prepared())
            .collect::<Option<Vec<_>>>()
            .expect("validated complete HLS objects");
        let keys = staged
            .iter()
            .map(|prepared| prepared.object.request_url.clone())
            .collect::<Vec<_>>();
        for prepared in staged {
            let key = prepared.object.request_url.clone();
            insert(&mut state, key, CachedHlsObject::from_prepared(prepared));
        }
        let revision = SegmentedAssetRevision::allocate(&mut state.last_asset_revision);
        let record = state
            .focus
            .get_mut(post)
            .expect("validated HLS focus record");
        record.objects = keys;
        record.snapshot.phase = SegmentedPhase::Ready;
        record.snapshot.eta_ms = Some(0);
        record.snapshot.detail = None;
        record.snapshot.authority = Some(HlsPreparedAssetAuthority::new(
            post.clone(),
            record.representation_id.clone(),
            revision,
        ));
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
        record.preparing = None;
        record.root_source = None;
        record.reserved_bytes = 0;
        record.assembly_bytes = 0;
        record.snapshot = SegmentedSnapshot::default();
        drop(state);
        self.changed.notify_waiters();
        true
    }

    pub(crate) fn restart_stage_object(&self, post: &PostId, generation: u64, url: &str) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation != generation || record.snapshot.phase == SegmentedPhase::Ready {
            return false;
        }
        record.staged.retain(|object| object.request_url() != url);
        record.preparing = None;
        record.reserved_bytes = 0;
        record.assembly_bytes = 0;
        record.snapshot.bytes_present = record.staged.iter().map(|object| object.len()).sum();
        record.snapshot.phase = SegmentedPhase::Queued;
        record.snapshot.eta_ms = None;
        record.snapshot.detail = None;
        drop(state);
        self.changed.notify_waiters();
        true
    }

    pub(crate) fn release_stage_attempt(&self, post: &PostId, generation: u64) -> bool {
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            return false;
        };
        if record.generation == generation
            && record.preparing.is_none()
            && record.snapshot.phase == SegmentedPhase::Queued
        {
            return true;
        }
        if record.generation != generation
            || record.preparing.is_some()
            || record.snapshot.phase != SegmentedPhase::Preparing
        {
            return false;
        }
        record.reserved_bytes = 0;
        record.assembly_bytes = 0;
        record.snapshot.phase = SegmentedPhase::Queued;
        record.snapshot.eta_ms = None;
        record.snapshot.detail = None;
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
        record.preparing = None;
        if phase == SegmentedPhase::Failed {
            record.staged.clear();
            record.root_source = None;
            record.reserved_bytes = 0;
            record.assembly_bytes = 0;
            record.snapshot.bytes_present = 0;
        }
        record.snapshot.eta_ms = (phase == SegmentedPhase::Ready).then_some(0);
        record.snapshot.detail = detail;
        drop(state);
        self.changed.notify_waiters();
        true
    }
}

#[cfg(test)]
#[path = "staged_axiom_test.rs"]
pub(crate) mod axiom_test_support;
