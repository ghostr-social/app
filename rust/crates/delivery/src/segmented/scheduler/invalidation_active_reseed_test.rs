use super::invalidation_active_reseed_fixture::{active, cancelled, focus, ready_root, root};
use super::{progress::Pending, SegmentedDelivery};
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;

#[tokio::test]
async fn invalidation_during_active_work_reseeds_only_the_affected_post() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus());
    let post = PostId::new("stream");
    let other = delivery.pending[&PostId::new("other")].clone();
    let generation = ready_root(&cache, &post);
    delivery.pending.remove(&post);
    delivery.active.insert(post.clone(), active());

    assert!(cache.invalidate_generation(root("stream").as_str(), generation));
    delivery.reseed_invalidated();
    assert!(delivery.active[&post].cancelling);
    assert_eq!(
        delivery.pending[&post],
        Pending::root(1, 3, 0, root("stream"))
    );
    assert_eq!(delivery.pending[&PostId::new("other")], other);

    delivery
        .finish(cancelled(post.clone()))
        .expect("valid test fixture");
    assert_eq!(
        delivery.pending[&post],
        Pending::root(1, 3, 0, root("stream"))
    );
}
