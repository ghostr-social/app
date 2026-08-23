use super::cancellation_race_test::{completed_active, focus};
use super::cancellation_success_fixture::{succeeded, MANIFEST};
use super::SegmentedDelivery;
use crate::segmented::{SegmentedCache, SegmentedPhase};
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};
use ghostr_engine::origin_model::OriginOutcome;
use ghostr_engine::PostId;

#[tokio::test]
async fn queued_success_is_superseded_without_losing_origin_or_resource_truth() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, "https://old.example/root.m3u8"));
    let post = PostId::new("stream");
    let done = succeeded(&cache, post.clone()).await;
    delivery.active.insert(post.clone(), completed_active());

    delivery.apply_focus(&focus(2, "https://new.example/root.m3u8"));
    assert!(!delivery.active[&post].cancelling);
    let finish = delivery.finish(done).unwrap();

    assert_eq!(finish.outcome, DecisionOutcome::Superseded);
    assert_eq!(
        finish.actual_resources,
        Some(ResourceCost::new(MANIFEST.len() as u64, 0, 0, 1))
    );
    assert_eq!(finish.observation.unwrap().outcome, OriginOutcome::Success);
    assert_eq!(cache.snapshot("stream").phase, SegmentedPhase::Queued);
    assert_eq!(delivery.pending[&post].generation, 2);
}
