use super::stage_capacity_fixture::{object, MIB};
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::PreparedComplete;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn stale_admission_cannot_evict_a_reclaimable_ready_bootstrap() {
    let cache = SegmentedCache::new();
    let old = PostId::new("old");
    let current = PostId::new("current");
    cache.replace_focus_window(
        2,
        vec![
            (old.clone(), vec![url("old-0")]),
            (current.clone(), vec![url("current")]),
        ],
        &HashSet::from([current.clone()]),
    );
    store_ready(&cache, &old);
    assert_eq!(cache.physical_available_bytes(), 7 * MIB as u64);

    assert!(cache
        .admit_stage(admission(&current, 1, "current", 8 * MIB))
        .is_none());
    assert_eq!(cache.snapshot("old").phase, SegmentedPhase::Ready);
    assert!(cache.object(&url("old-0")).is_some());
}

fn store_ready(cache: &SegmentedCache, post: &PostId) {
    for (attempt, bytes) in [MIB, 8 * MIB, 8 * MIB, 8 * MIB].into_iter().enumerate() {
        let name = format!("old-{attempt}");
        let lease = cache.admit_stage(admission(post, 2, &name, bytes)).unwrap();
        assert!(lease.commit_complete(PreparedComplete::new(object(&name, bytes))));
    }
    assert!(cache.mark_stage_ready(post, 2));
}

fn admission(post: &PostId, generation: u64, name: &str, bytes: usize) -> StageAdmission {
    let request = StageRequest::new(url(name), 0, bytes as u64);
    StageAdmission::new(
        post.clone(),
        StageFence::new(generation, 1, request),
        500,
        StageReservation::block(bytes as u64),
    )
}

fn url(name: &str) -> String {
    format!("https://example.com/{name}")
}
