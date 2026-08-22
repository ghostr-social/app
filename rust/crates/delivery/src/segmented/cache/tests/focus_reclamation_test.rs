use super::{prepared, store_ready};
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::PostId;

#[test]
fn replacing_focus_reclaims_unreferenced_ready_objects_before_reservation() {
    let cache = SegmentedCache::new();
    let old = [PostId::new("old-a"), PostId::new("old-b")];
    cache.replace_focus(
        1,
        old.iter()
            .map(|post| (post.clone(), vec![post.as_str().to_owned()]))
            .collect(),
    );
    for post in &old {
        store_ready(&cache, post, 1, prepared(post.as_str()));
    }
    assert_eq!(cache.planning_available_bytes(), 0);

    let current = PostId::new("current");
    cache.replace_focus(2, vec![(current.clone(), vec!["current".to_owned()])]);

    assert_eq!(cache.planning_available_bytes(), 32 * 1024 * 1024);
    assert!(cache.mark_stage_preparing(
        &current,
        2,
        500,
        HlsBootstrapStage::FirstSegment.maximum_bytes(),
    ));
}
