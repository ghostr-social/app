use super::stage_lease_fixture::{focused_cache, object, source, store_partial};
use crate::segmented::cache::{StageAdmission, StageFence, StageRequest, StageReservation};
use ghostr_engine::PostId;
use std::time::Duration;

#[tokio::test]
async fn final_stage_memory_stays_counted_after_focus_removal() {
    let post = PostId::new("current");
    let cache = focused_cache(&post);
    store_partial(&cache, &post, 256 * 1024);
    let request = StageRequest::new(source(), 256 * 1024, 128 * 1024);
    let fence = StageFence::new(1, 7, request);
    let reservation = StageReservation::final_block(128 * 1024, 384 * 1024).unwrap();
    let admission = StageAdmission::new(post, fence, 500, reservation);

    let mut lease = cache.admit_stage(admission).expect("final stage admitted");
    assert_eq!(cache.physical_used_bytes(), 768 * 1024);
    let assembly = lease
        .claim_assembly(&object(128 * 1024))
        .expect("exact prefix claimed");
    cache.replace_focus(2, Vec::new());
    assert_eq!(cache.physical_used_bytes(), 768 * 1024);

    drop(assembly);
    let notifier = cache.notifier();
    let released = notifier.notified();
    drop(lease);
    tokio::time::timeout(Duration::from_millis(100), released)
        .await
        .expect("released stage capacity wakes planners");
    assert_eq!(cache.physical_used_bytes(), 0);
}
