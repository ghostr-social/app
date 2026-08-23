use super::stage_capacity_fixture::{object, MIB};
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::PreparedComplete;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn insufficient_reclaim_cannot_evict_a_ready_bootstrap() {
    let cache = SegmentedCache::new();
    let old = PostId::new("old");
    let held = PostId::new("held");
    let current = PostId::new("current");
    cache.replace_focus_window(
        1,
        [&old, &held, &current]
            .into_iter()
            .map(|post| (post.clone(), vec![url(post.as_str())]))
            .collect(),
        &HashSet::from([held.clone(), current.clone()]),
    );
    store_ready(&cache, &old, &[MIB]);
    store_ready(&cache, &held, &[MIB, 8 * MIB, 8 * MIB, 8 * MIB]);
    assert_eq!(cache.physical_available_bytes(), 6 * MIB as u64);

    assert!(cache
        .admit_stage(admission(current, "current", 8 * MIB))
        .is_none());
    assert_eq!(cache.snapshot("old").phase, SegmentedPhase::Ready);
    assert!(cache.object(&url("old-0")).is_some());
}

fn store_ready(cache: &SegmentedCache, post: &PostId, blocks: &[usize]) {
    for (index, bytes) in blocks.iter().copied().enumerate() {
        let name = format!("{}-{index}", post.as_str());
        let lease = cache
            .admit_stage(admission(post.clone(), &name, bytes))
            .expect("bootstrap stage admitted");
        assert!(lease.commit_complete(PreparedComplete::new(object(&name, bytes))));
    }
    assert!(cache.mark_stage_ready(post, 1));
}

fn admission(post: PostId, name: &str, bytes: usize) -> StageAdmission {
    let request = StageRequest::new(url(name), 0, bytes as u64);
    StageAdmission::new(
        post,
        StageFence::new(1, 1, request),
        500,
        StageReservation::block(bytes as u64),
    )
}

fn url(name: &str) -> String {
    format!("https://example.com/{name}")
}
