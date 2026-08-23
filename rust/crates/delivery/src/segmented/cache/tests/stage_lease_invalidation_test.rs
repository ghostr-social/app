use super::stage_capacity_fixture::object;
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::prepare::PreparedComplete;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;

#[test]
fn invalidation_breaks_a_live_stage_fence_before_reseed() {
    let cache = SegmentedCache::new();
    let post = PostId::new("stream");
    let root = object("root", 4);
    cache.replace_focus(1, vec![(post.clone(), vec![root.request_url.clone()])]);
    let root_lease = cache.admit_stage(admission(&post, 1, "root", 4)).unwrap();
    assert!(root_lease.commit_complete(PreparedComplete::new(root.clone())));
    assert!(cache.mark_stage_ready(&post, 1));
    let generation = cache.object(&root.request_url).unwrap().generation();
    assert!(cache.reset_stage_retry(&post, 1));
    let stale = cache.admit_stage(admission(&post, 2, "next", 4)).unwrap();

    assert!(cache.invalidate_generation(&root.request_url, generation));
    let replacement = cache.admit_stage(admission(&post, 3, "next", 4)).unwrap();
    drop(stale);

    assert!(replacement.commit_complete(PreparedComplete::new(object("next", 4))));
}

fn admission(post: &PostId, attempt: u64, name: &str, bytes: u64) -> StageAdmission {
    let request = StageRequest::new(format!("https://example.com/{name}"), 0, bytes);
    StageAdmission::new(
        post.clone(),
        StageFence::new(1, attempt, request),
        500,
        StageReservation::block(bytes),
    )
}
