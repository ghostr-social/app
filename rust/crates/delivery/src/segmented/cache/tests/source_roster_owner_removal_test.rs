use super::super::{SegmentedCache, SegmentedPhase};
use super::source_roster_reuse_test::{object, roots, store_ready};
use ghostr_engine::PostId;

#[test]
fn removing_the_cache_owning_root_invalidates_ready_bootstrap() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    let primary = "https://primary.example/index.m3u8";
    let backup = "https://backup.example/index.m3u8";
    cache.replace_focus(1, vec![(post.clone(), roots(primary, backup))]);
    store_ready(&cache, &post, 1, object(backup));

    cache.replace_focus(2, vec![(post, vec![primary.to_owned()])]);

    assert_eq!(cache.snapshot("post").phase, SegmentedPhase::Queued);
    assert!(cache.object(backup).is_none());
}
