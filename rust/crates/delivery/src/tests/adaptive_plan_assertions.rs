use crate::manager::plan::PlannedWork;
use ghostr_engine::PostId;
use std::collections::HashSet;

pub(super) fn posts(work: &PlannedWork) -> HashSet<PostId> {
    work.transfers
        .iter()
        .map(|transfer| transfer.request.chunk.post.clone())
        .collect()
}
