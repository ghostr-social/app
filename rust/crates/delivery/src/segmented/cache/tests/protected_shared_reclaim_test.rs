use super::store_ready;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::adaptive::HlsBootstrapStage;
use ghostr_engine::PostId;
use std::collections::HashSet;
use std::sync::Arc;

const MIB: usize = 1024 * 1024;
const SHARED: &str = "https://cache.example/shared";
const EXCLUSIVE: &str = "https://cache.example/exclusive";

#[test]
fn pressure_reclaim_keeps_keys_shared_with_a_protected_ready_bootstrap() {
    let cache = SegmentedCache::new();
    let old = PostId::new("old");
    let protected = PostId::new("protected");
    let current = PostId::new("current");
    let protected_set = HashSet::from([protected.clone(), current.clone()]);
    cache.replace_focus_window(
        1,
        [&old, &protected, &current]
            .into_iter()
            .map(|post| (post.clone(), vec![post.as_str().to_owned()]))
            .collect(),
        &protected_set,
    );
    store_ready(
        &cache,
        &old,
        1,
        vec![object(SHARED, MIB), object(EXCLUSIVE, 17 * MIB)],
    );
    store_ready(&cache, &protected, 1, vec![object(SHARED, MIB)]);
    stage_current(&cache, &current);

    assert_eq!(cache.physical_available_bytes(), 4 * MIB as u64);
    assert_eq!(cache.planning_available_bytes(), 21 * MIB as u64);
    assert!(cache.mark_stage_preparing(
        &current,
        1,
        500,
        HlsBootstrapStage::FirstSegment.maximum_bytes(),
    ));

    assert_eq!(cache.snapshot("old").phase, SegmentedPhase::Queued);
    assert_eq!(cache.snapshot("protected").phase, SegmentedPhase::Ready);
    assert!(cache.object(SHARED).is_some());
    assert!(cache.object(EXCLUSIVE).is_none());
}

fn stage_current(cache: &SegmentedCache, post: &PostId) {
    for (index, bytes) in [MIB, MIB, 8 * MIB].into_iter().enumerate() {
        assert!(cache.mark_stage_preparing(post, 1, 500, bytes as u64));
        let key = format!("https://current.example/{index}");
        assert!(cache
            .store_stage_object(post, 1, object(&key, bytes))
            .is_some());
    }
}

fn object(request_url: &str, bytes: usize) -> PreparedObject {
    PreparedObject {
        request_url: request_url.to_owned(),
        final_url: request_url.parse().unwrap(),
        body: Arc::from(vec![0; bytes]),
        content_type: None,
    }
}
