use crate::manager::timeline::axiom_test_support::TimelineIncomplete;
use crate::manager::timeline::axiom_test_support::TimelineInput;
use crate::manager::timeline::axiom_test_support::TimelineParse;
use crate::manager::timeline::axiom_test_support::TimelineParser;
use crate::manager::timeline::{
    TimelineCoordinator, TimelineEvidence, TimelineJobOutcome, TimelineSchedule, TimelineTerminal,
};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::support::temp_directory;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use ghostr_engine::media_timeline::TimelineParseControl;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test(start_paused = true)]
async fn worker_failure_retries_after_backoff_without_self_waking_the_manager() {
    let root = temp_directory("timeline-retry-backoff");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let binding = binding(&catalog(&["post"]), "post");
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
    let snapshot = store
        .media_snapshot("post")
        .await
        .expect("valid test fixture");
    let evidence =
        TimelineEvidence::from_snapshot(&binding, &snapshot).expect("valid test fixture");
    let parser = Arc::new(FailsOnce::default());
    let mut coordinator =
        TimelineCoordinator::with_parser(store, std::sync::Arc::<FailsOnce>::clone(&parser), 1);
    let post = PostId::new("post");
    assert_eq!(
        coordinator.schedule(post.clone(), evidence.clone()),
        TimelineSchedule::Started
    );
    coordinator.dispatch(core::slice::from_ref(&post));

    assert!(
        tokio::time::timeout(Duration::from_millis(50), coordinator.recv())
            .await
            .is_err()
    );
    tokio::time::advance(Duration::from_millis(50)).await;
    let result = coordinator.recv().await.expect("valid test fixture");

    assert!(matches!(
        coordinator.validate(result, Some(&evidence)),
        Some(TimelineJobOutcome::Terminal(TimelineTerminal::Incomplete(
            TimelineIncomplete::Unavailable
        )))
    ));
    assert_eq!(parser.calls.load(Ordering::Acquire), 2);
    tokio::fs::remove_dir_all(root)
        .await
        .expect("valid test fixture");
}

#[derive(Default)]
struct FailsOnce {
    calls: AtomicUsize,
}

impl TimelineParser for FailsOnce {
    fn parse(&self, _input: TimelineInput, _control: &dyn TimelineParseControl) -> TimelineParse {
        assert!(
            self.calls.fetch_add(1, Ordering::AcqRel) != 0,
            "injected parser worker failure"
        );
        TimelineParse::Completed(TimelineTerminal::Incomplete(
            TimelineIncomplete::Unavailable,
        ))
    }
}
