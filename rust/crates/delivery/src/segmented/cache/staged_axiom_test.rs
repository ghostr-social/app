use super::*;

use super::super::capacity::fits;

use super::super::StageReservation;

impl SegmentedCache {
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
            super::super::objects::reclaim_unprotected_ready(&mut state);
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
}
