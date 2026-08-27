use crate::manager::plan::PlannedWork;
use ghostr_engine::PostId;
use std::collections::HashSet;

pub(super) fn allocated_posts(work: &PlannedWork) -> HashSet<PostId> {
    work.plan
        .allocations
        .iter()
        .map(|allocation| allocation.post.clone())
        .collect()
}
