use super::cancellation_race_test::{
    completed_active as failed_active, failed, focus as failed_focus,
};
use super::cancellation_race_test::{completed_active, focus};
use super::cancellation_success_fixture::succeeded;
use super::SegmentedDelivery;
use crate::segmented::scheduler::SegmentedRecovery;
use crate::segmented::SegmentedCache;
use ghostr_engine::adaptive::DecisionOutcome;
use ghostr_engine::PostId;

const A: &str = "https://old.example/root.m3u8";
const B: &str = "https://new.example/root.m3u8";

#[tokio::test]
async fn focus_round_trip_keeps_fresh_root_while_old_terminal_is_queued() {
    let cache = SegmentedCache::new();
    let mut delivery = SegmentedDelivery::new(cache.clone());
    delivery.apply_focus(&focus(1, A));
    let post = PostId::new("stream");
    let done = succeeded(&cache, post.clone()).await;
    delivery.active.insert(post.clone(), completed_active());

    delivery.apply_focus(&focus(2, B));
    assert_eq!(delivery.pending[&post].generation, 2);
    delivery.apply_focus(&focus(3, A));
    assert_eq!(delivery.pending[&post].generation, 3);

    let finish = delivery.finish(done).unwrap();
    assert_eq!(finish.outcome, DecisionOutcome::Superseded);
    assert_eq!(delivery.pending[&post].generation, 3);
    assert_eq!(delivery.pending[&post].root_source, A);
}

#[tokio::test]
async fn stale_round_trip_failure_cannot_retry_the_fresh_generation() {
    let mut delivery = SegmentedDelivery::new(SegmentedCache::new());
    delivery.apply_focus(&failed_focus(1, A));
    let post = PostId::new("stream");
    delivery.active.insert(post.clone(), failed_active());
    delivery.apply_focus(&failed_focus(2, B));
    delivery.apply_focus(&failed_focus(3, A));
    assert_eq!(delivery.pending[&post].generation, 3);

    let finish = delivery.finish(failed(post.clone())).unwrap();
    assert!(matches!(finish.recovery, SegmentedRecovery::None));
    assert_eq!(delivery.pending[&post].generation, 3);
}
