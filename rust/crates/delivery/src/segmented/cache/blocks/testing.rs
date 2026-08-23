use super::{StageBlock, StoredStage};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;

impl SegmentedCache {
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
