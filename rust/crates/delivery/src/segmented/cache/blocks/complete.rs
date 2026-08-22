use super::super::SegmentedCache;
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::PostId;

pub(in crate::segmented) struct CompleteStage {
    pub object: PreparedObject,
    pub(super) offset: u64,
    pub(super) block_bytes: u64,
    pub(super) reservation: Option<AssemblyReservation>,
}

pub(super) struct AssemblyReservation {
    cache: SegmentedCache,
    post: PostId,
    generation: u64,
    bytes: u64,
}

impl AssemblyReservation {
    pub(super) fn new(cache: SegmentedCache, post: PostId, generation: u64, bytes: u64) -> Self {
        Self {
            cache,
            post,
            generation,
            bytes,
        }
    }
}

impl Drop for AssemblyReservation {
    fn drop(&mut self) {
        self.cache
            .release_stage_assembly(&self.post, self.generation, self.bytes);
    }
}
