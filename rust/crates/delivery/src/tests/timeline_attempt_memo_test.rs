use crate::manager::timeline::axiom_test_support::TimelineAttemptDisposition;
use crate::manager::timeline::axiom_test_support::TimelineAttempts;
use crate::manager::timeline::TimelineEvidence;
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
    store
        .bind_representation(binding.clone())
        .await
        .expect("valid test fixture");
    store
        .set_total_len("post", 32)
        .await
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"abcdefgh")
        .await
        .expect("valid test fixture");
    let first = evidence(&store, &binding).await;
    let mut attempts = TimelineAttempts::default();

    let first_attempt = attempts
        .start(post.clone(), first.clone())
        .expect("valid test fixture");
    assert!(attempts.start(post.clone(), first.clone()).is_none());
    assert!(attempts.finish(&first_attempt, TimelineAttemptDisposition::Terminal));
    assert!(attempts.start(post.clone(), first).is_none());

    store
        .write_range("post", 0, b"ABCDEFGH")
        .await
        .expect("valid test fixture");
    let second = attempts
        .start(post.clone(), evidence(&store, &binding).await)
        .expect("valid test fixture");
    store
        .write_range("post", 0, b"12345678")
        .await
        .expect("valid test fixture");
    let third = attempts
        .start(post, evidence(&store, &binding).await)
        .expect("valid test fixture");

    assert!(!attempts.finish(&second, TimelineAttemptDisposition::Terminal));
    assert!(attempts.finish(&third, TimelineAttemptDisposition::Terminal));
    tokio::fs::remove_dir_all(root)
        .await
        .expect("valid test fixture");
}

async fn evidence(
    store: &PartialRangeStore,
    binding: &ghostr_engine::representation::RepresentationBinding,
) -> TimelineEvidence {
    let snapshot = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    TimelineEvidence::from_snapshot(binding, &snapshot).expect("valid test fixture")
}
