use crate::manager::timeline::{TimelineAttemptDisposition, TimelineAttempts, TimelineEvidence};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::support::temp_directory;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn terminal_attempts_are_exactly_memoized_and_stale_results_are_rejected() {
    let root = temp_directory("timeline-attempt-memo");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    let catalog = catalog(&["post"]);
    let binding = binding(&catalog, "post");
    let post = PostId::new("post");
    store.bind_representation(binding.clone()).await.unwrap();
    store.set_total_len("post", 32).await.unwrap();
    store.write_range("post", 0, b"abcdefgh").await.unwrap();
    let first = evidence(&store, &binding).await;
    let mut attempts = TimelineAttempts::default();

    let first_attempt = attempts.start(post.clone(), first.clone()).unwrap();
    assert!(attempts.start(post.clone(), first.clone()).is_none());
    assert!(attempts.finish(&first_attempt, TimelineAttemptDisposition::Terminal));
    assert!(attempts.start(post.clone(), first).is_none());

    store.write_range("post", 0, b"ABCDEFGH").await.unwrap();
    let second = attempts
        .start(post.clone(), evidence(&store, &binding).await)
        .unwrap();
    store.write_range("post", 0, b"12345678").await.unwrap();
    let third = attempts
        .start(post, evidence(&store, &binding).await)
        .unwrap();

    assert!(!attempts.finish(&second, TimelineAttemptDisposition::Terminal));
    assert!(attempts.finish(&third, TimelineAttemptDisposition::Terminal));
    tokio::fs::remove_dir_all(root).await.unwrap();
}

async fn evidence(
    store: &PartialRangeStore,
    binding: &ghostr_engine::representation::RepresentationBinding,
) -> TimelineEvidence {
    let snapshot = store.media_snapshot("post").await.unwrap();
    TimelineEvidence::from_snapshot(binding, &snapshot).unwrap()
}
