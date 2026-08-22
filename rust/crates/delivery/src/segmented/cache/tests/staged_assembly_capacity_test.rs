use crate::segmented::cache::{StageBlock, StageReservation, StoredStage};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

const MIB: usize = 1024 * 1024;

#[test]
fn completed_assembly_is_counted_until_its_result_is_dropped() {
    let (cache, post) = cache_with_ready_bytes(16 * MIB);
    store_partial(&cache, &post, 4 * MIB);
    let reservation = StageReservation::final_block(4 * MIB as u64, 8 * MIB as u64).unwrap();
    assert!(cache.mark_stage_preparing(&post, 1, 500, reservation));
    let Some(StoredStage::Complete(completed)) = cache.store_stage_block(
        &post,
        1,
        StageBlock::complete(4 * MIB as u64, object("current", 4 * MIB)),
    ) else {
        panic!("assembly fits the cache");
    };

    assert_eq!(cache.physical_available_bytes(), 0);
    assert_eq!(cache.physical_used_bytes(), 32 * MIB as u64);
    drop(completed);
    assert_eq!(cache.physical_available_bytes(), 8 * MIB as u64);
    assert!(cache.release_stage_attempt(&post, 1));
    assert_eq!(cache.physical_available_bytes(), 12 * MIB as u64);
}

#[test]
fn final_assembly_is_rejected_before_exceeding_the_cache_limit() {
    let (cache, post) = cache_with_ready_bytes(16 * MIB);
    store_partial(&cache, &post, 8 * MIB);
    let reservation = StageReservation::final_block(4 * MIB as u64, 12 * MIB as u64).unwrap();
    assert!(!cache.mark_stage_preparing(&post, 1, 500, reservation));
    assert_eq!(cache.physical_available_bytes(), 8 * MIB as u64);
}

fn cache_with_ready_bytes(bytes: usize) -> (SegmentedCache, PostId) {
    let cache = SegmentedCache::new();
    let held = PostId::new("held");
    let current = PostId::new("current");
    cache.replace_focus(
        1,
        vec![
            (held.clone(), vec!["held".into()]),
            (current.clone(), vec!["current".into()]),
        ],
    );
    assert!(cache.mark_stage_preparing(&held, 1, 500, bytes as u64));
    cache
        .store_stage_object(&held, 1, object("held", bytes))
        .unwrap();
    assert!(cache.mark_stage_ready(&held, 1));
    (cache, current)
}

fn store_partial(cache: &SegmentedCache, post: &PostId, bytes: usize) {
    assert!(cache.mark_stage_preparing(post, 1, 500, bytes as u64));
    assert!(matches!(
        cache.store_stage_block(post, 1, StageBlock::partial(0, object("current", bytes))),
        Some(StoredStage::Partial)
    ));
}

fn object(request_url: &str, bytes: usize) -> PreparedObject {
    PreparedObject {
        request_url: request_url.to_owned(),
        final_url: Url::parse(&format!("https://example.com/{request_url}")).unwrap(),
        body: Arc::from(vec![0; bytes]),
        content_type: None,
        cache: Default::default(),
    }
}
