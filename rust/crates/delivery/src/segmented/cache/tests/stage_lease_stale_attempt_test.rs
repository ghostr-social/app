use super::stage_lease_fixture::{focused_cache, source};
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use crate::segmented::SegmentedPhase;
use ghostr_engine::PostId;

#[test]
fn stale_lease_drop_cannot_release_same_generation_retry() {
    let post = PostId::new("current");
    let cache = focused_cache(&post);
    let old = admission(&post, 7);
    let old_lease = cache.admit_stage(old).expect("old attempt admitted");
    cache.replace_focus(1, vec![(post.clone(), vec![source()])]);
    let current = admission(&post, 8);
    let current_lease = cache.admit_stage(current).expect("retry admitted");

    drop(old_lease);
    assert_eq!(cache.snapshot("current").phase, SegmentedPhase::Preparing);
    assert_eq!(cache.physical_used_bytes(), 128 * 1024);

    drop(current_lease);
    assert_eq!(cache.snapshot("current").phase, SegmentedPhase::Queued);
    assert_eq!(cache.physical_used_bytes(), 0);
}

fn admission(post: &PostId, attempt: u64) -> StageAdmission {
    let request = StageRequest::new(source(), 0, 128 * 1024);
    let fence = StageFence::new(1, attempt, request);
    StageAdmission::new(
        post.clone(),
        fence,
        500,
        StageReservation::block(128 * 1024),
    )
}
