use super::{FocusRecord, SegmentedCache, SegmentedPhase};
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::PostId;

mod commit;
use commit::{accepts_complete, commit_complete, commit_partial, CompleteValidation};
mod complete;
use complete::AssemblyReservation;
pub(in crate::segmented) use complete::CompleteStage;
mod input;
pub(in crate::segmented) use input::StageBlock;

pub(in crate::segmented) enum StoredStage {
    Partial,
    Complete(Box<CompleteStage>),
}

impl SegmentedCache {
    pub(in crate::segmented) fn store_stage_block(
        &self,
        post: &PostId,
        generation: u64,
        block: StageBlock,
    ) -> Option<StoredStage> {
        let mut state = self.lock();
        let stored = self.store_locked(&mut state, post, generation, block)?;
        let notify = matches!(stored, StoredStage::Partial);
        drop(state);
        if notify {
            self.changed.notify_waiters();
        }
        Some(stored)
    }

    fn store_locked(
        &self,
        state: &mut super::CacheState,
        post: &PostId,
        generation: u64,
        block: StageBlock,
    ) -> Option<StoredStage> {
        let bytes = block.object.body.len() as u64;
        if !accepts(state.focus.get(post)?, generation, bytes) {
            return None;
        }
        match block.complete {
            true => self.complete_locked(state, post, generation, block),
            false => partial_locked(state, post, block),
        }
    }

    fn complete_locked(
        &self,
        state: &mut super::CacheState,
        post: &PostId,
        generation: u64,
        block: StageBlock,
    ) -> Option<StoredStage> {
        let StageBlock { offset, object, .. } = block;
        let block_bytes = object.body.len() as u64;
        let record = state.focus.get(post)?;
        if offset == 0 && record.assembly_bytes == 0 {
            return Some(StoredStage::Complete(Box::new(CompleteStage {
                object,
                offset,
                block_bytes,
                reservation: None,
            })));
        }
        let total = continuation_total(record, offset, &object)?;
        if record.assembly_bytes != total {
            return None;
        }
        let object =
            assemble(state, post, offset, object).expect("validated HLS continuation assembly");
        let reservation = AssemblyReservation::new(self.clone(), post.clone(), generation, total);
        Some(StoredStage::Complete(Box::new(CompleteStage {
            object,
            offset,
            block_bytes,
            reservation: Some(reservation),
        })))
    }

    pub(in crate::segmented) fn commit_stage_complete(
        &self,
        post: &PostId,
        generation: u64,
        completed: CompleteStage,
    ) -> bool {
        let CompleteStage {
            object,
            offset,
            block_bytes,
            reservation,
        } = completed;
        let mut state = self.lock();
        let Some(record) = state.focus.get_mut(post) else {
            drop(state);
            drop(object);
            drop(reservation);
            return false;
        };
        let validation = CompleteValidation::new(generation, offset, block_bytes, &object);
        if !accepts_complete(record, validation) {
            drop(state);
            drop(object);
            drop(reservation);
            return false;
        }
        commit_complete(record, offset, object);
        drop(state);
        drop(reservation);
        self.changed.notify_waiters();
        true
    }

    fn release_stage_assembly(&self, post: &PostId, generation: u64, bytes: u64) {
        let mut state = self.lock();
        let released = state.focus.get_mut(post).is_some_and(|record| {
            if record.generation != generation || record.assembly_bytes != bytes {
                return false;
            }
            record.assembly_bytes = 0;
            true
        });
        drop(state);
        if released {
            self.changed.notify_waiters();
        }
    }

    #[cfg(test)]
    pub(in crate::segmented) fn store_stage_object(
        &self,
        post: &PostId,
        generation: u64,
        object: PreparedObject,
    ) -> Option<u64> {
        let StoredStage::Complete(completed) =
            self.store_stage_block(post, generation, StageBlock::complete(0, object))?
        else {
            return None;
        };
        self.commit_stage_complete(post, generation, *completed)
            .then(|| self.snapshot(post.as_str()).bytes_present)
    }
}

fn partial_locked(
    state: &mut super::CacheState,
    post: &PostId,
    block: StageBlock,
) -> Option<StoredStage> {
    let record = state.focus.get_mut(post)?;
    if record.assembly_bytes != 0 {
        return None;
    }
    commit_partial(record, block.offset, block.object)?;
    Some(StoredStage::Partial)
}

fn accepts(record: &FocusRecord, generation: u64, bytes: u64) -> bool {
    record.generation == generation
        && record.snapshot.phase != SegmentedPhase::Ready
        && bytes > 0
        && bytes <= record.reserved_bytes
}

fn continuation_total(record: &FocusRecord, offset: u64, block: &PreparedObject) -> Option<u64> {
    record
        .staged
        .iter()
        .find(|known| known.request_url() == block.request_url)
        .and_then(|known| known.continuation_len(block, offset))
}

fn assemble(
    state: &super::CacheState,
    post: &PostId,
    offset: u64,
    block: PreparedObject,
) -> Option<PreparedObject> {
    state
        .focus
        .get(post)?
        .staged
        .iter()
        .find(|known| known.request_url() == block.request_url)?
        .assembled_with(block, offset)
}
