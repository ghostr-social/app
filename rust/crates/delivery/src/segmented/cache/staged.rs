#[cfg(test)]
use super::capacity::fits;
use super::objects::insert;
#[cfg(test)]
use super::StageReservation;
use super::{CachedHlsObject, SegmentedCache, SegmentedPhase, SegmentedSnapshot};
use ghostr_engine::PostId;

impl SegmentedCache {
    #[cfg(test)]
    pub(crate) fn mark_stage_preparing<R: Into<StageReservation>>(
        &self,
        post: &PostId,
        generation: u64,
        eta_ms: u64,
        reservation: R,
    ) -> bool {
        let reservation = reservation.into();
        let Some(total_bytes) = reservation.total_bytes() else {
            return false;
        };
        if reservation.block_bytes == 0 {
            return false;
        }
        let mut state = self.lock();
        let Some(record) = state.focus.get(post) else {
            return false;
        };
        if record.generation != generation
            || record.snapshot.phase == SegmentedPhase::Ready
            || record.preparing.is_some()
            || record.assembly_bytes != 0
        {
            return false;
        }
        let protected = record.protected;
        if !fits(&state, post, total_bytes) && protected {
            super::objects::reclaim_unprotected_ready(&mut state);
        }
        if !fits(&state, post, total_bytes) {
            return false;
        }
        let record = state
            .focus
            .get_mut(post)
            .expect("validated HLS cache focus");
        record.snapshot.phase = SegmentedPhase::Preparing;
        record.snapshot.eta_ms = Some(eta_ms);
        record.snapshot.detail = None;
        record.reserved_bytes = reservation.block_bytes;
        record.assembly_bytes = reservation.assembly_bytes;
        drop(state);
        self.changed.notify_waiters();
        true
    }

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
        let staged = std::mem::take(&mut record.staged);
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
