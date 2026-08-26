use crate::manager::timeline::axiom_test_support::TimelineIncomplete;
use crate::manager::timeline::{TimelineCoordinator, TimelineEvidence, TimelineJobOutcome, TimelineSchedule, TimelineTerminal};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::support::temp_directory;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn changed_evidence_runs_without_waiting_and_rejects_the_stale_result() {
    let root = temp_directory("timeline-coordinator-stale");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let binding = binding(&catalog(&["post"]), "post");
    store.bind_representation(binding.clone()).await.expect("valid test fixture");
    store.set_total_len("post", 32).await.expect("valid test fixture");
    store.write_range("post", 0, b"abcdefgh").await.expect("valid test fixture");
    let (parser, mut started) = GatedTimelineParser::new(None, 2);
    let mut coordinator = TimelineCoordinator::with_parser(std::sync::Arc::clone(&store), std::sync::Arc::<GatedTimelineParser>::clone(&parser), 2);
    let post = PostId::new("post");

    let first = evidence(&store, &binding).await;
    assert_eq!(
        coordinator.schedule(post.clone(), first),
        TimelineSchedule::Started
    );
    coordinator.dispatch(core::slice::from_ref(&post));
    assert_eq!(started.recv().await, Some(0));
    store.write_range("post", 0, b"ABCDEFGH").await.expect("valid test fixture");
    let second = evidence(&store, &binding).await;
    assert_eq!(
        coordinator.schedule(post.clone(), second.clone()),
        TimelineSchedule::Started
    );
    coordinator.dispatch(core::slice::from_ref(&post));
    assert_eq!(started.recv().await, Some(1));

    parser.release(0);
    let stale = coordinator.recv().await.expect("valid test fixture");
    assert!(coordinator.validate(stale, Some(&second)).is_none());
    parser.release(1);
    let current = coordinator.recv().await.expect("valid test fixture");
    assert!(matches!(
        coordinator.validate(current, Some(&second)),
        Some(TimelineJobOutcome::Terminal(TimelineTerminal::Incomplete(
            TimelineIncomplete::Unavailable
        )))
    ));
    tokio::fs::remove_dir_all(root).await.expect("valid test fixture");
}

async fn evidence(
    store: &PartialRangeStore,
    binding: &ghostr_engine::representation::RepresentationBinding,
) -> TimelineEvidence {
    TimelineEvidence::from_snapshot(binding, &store.media_snapshot("post").await.expect("valid test fixture")).expect("valid test fixture")
}
