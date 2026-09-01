use crate::manager::timeline::axiom_test_support::TimelineInput;
use crate::manager::timeline::axiom_test_support::TimelineParse;
use crate::manager::timeline::axiom_test_support::TimelineParser;
use crate::manager::timeline::{
    TimelineCoordinator, TimelineEvidence, TimelineJobOutcome, TimelineSchedule,
};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::support::temp_directory;
use core::time::Duration;
use ghostr_engine::media_timeline::TimelineParseControl;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn a_current_cancelled_outcome_is_superseded_and_never_memoized() {
    let (mut coordinator, evidence, post, root) = fixture().await;
    assert_eq!(
        coordinator.schedule(post.clone(), evidence.clone()),
        TimelineSchedule::Started
    );
    coordinator.dispatch(core::slice::from_ref(&post));
    let result = next_result(&mut coordinator).await;
    assert!(matches!(
        coordinator.validate(result, Some(&evidence)),
        Some(TimelineJobOutcome::Superseded)
    ));
    assert_eq!(
        coordinator.schedule(post, evidence),
        TimelineSchedule::Started
    );
    tokio::fs::remove_dir_all(root)
        .await
        .expect("valid test fixture");
}

async fn fixture() -> (TimelineCoordinator, TimelineEvidence, PostId, PathBuf) {
    let root = temp_directory("timeline-cancelled-outcome");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let catalog = catalog(&["post"]);
    let binding = binding(&catalog, "post");
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
    let coordinator = TimelineCoordinator::with_parser(store, Arc::new(CancelledParser), 1);
    (coordinator, evidence, PostId::new("post"), root)
}

async fn next_result(
    coordinator: &mut TimelineCoordinator,
) -> crate::manager::timeline::TimelineResult {
    tokio::time::timeout(Duration::from_secs(1), coordinator.recv())
        .await
        .expect("valid test fixture")
        .expect("valid test fixture")
}

struct CancelledParser;

impl TimelineParser for CancelledParser {
    fn parse(&self, _input: TimelineInput, _control: &dyn TimelineParseControl) -> TimelineParse {
        TimelineParse::Cancelled
    }
}
