use crate::manager::timeline::{TimelineCoordinator, TimelineEvidence, TimelineSchedule};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::support::temp_directory;
use crate::tests::timeline_parser_fixture::GatedTimelineParser;
use core::time::Duration;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn newly_current_pending_timeline_is_dispatched_before_old_prefetch() {
    let root = temp_directory("timeline-priority");
    let store = Arc::new(PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let catalog = catalog(&["a", "b", "c"]);
    for post in ["a", "b", "c"] {
        store
            .bind_representation(binding(&catalog, post))
            .await
            .expect("valid test fixture");
        store
            .set_total_len(post, 32)
            .await
            .expect("valid test fixture");
        store
            .write_range(post, 0, post.as_bytes())
            .await
            .expect("valid test fixture");
    }
    let (parser, mut started) = GatedTimelineParser::new(None, 3);
    let _release = ReleaseAll(std::sync::Arc::clone(&parser));
    let mut coordinator = TimelineCoordinator::with_parser(
        std::sync::Arc::clone(&store),
        std::sync::Arc::<GatedTimelineParser>::clone(&parser),
        1,
    );
    let posts = [PostId::new("a"), PostId::new("b"), PostId::new("c")];

    for post in &posts {
        assert_eq!(
            coordinator.schedule(post.clone(), evidence(&store, &catalog, post).await),
            TimelineSchedule::Started
        );
    }
    coordinator.dispatch(&posts);
    assert_eq!(recv(&mut started).await, 0);
    parser.release(0);
    let _ = coordinator.recv().await.expect("valid test fixture");
    coordinator.dispatch(&[posts[2].clone(), posts[1].clone()]);
    assert_eq!(recv(&mut started).await, 1);
    parser.release(1);
    let next = coordinator.recv().await.expect("valid test fixture");

    assert_eq!(next.post(), &posts[2]);
    tokio::fs::remove_dir_all(root)
        .await
        .expect("valid test fixture");
}

struct ReleaseAll(Arc<GatedTimelineParser>);

impl Drop for ReleaseAll {
    fn drop(&mut self) {
        for gate in 0..3 {
            self.0.release(gate);
        }
    }
}

async fn evidence(
    store: &PartialRangeStore,
    catalog: &ghostr_engine::catalog::Catalog,
    post: &PostId,
) -> TimelineEvidence {
    let binding = binding(catalog, post.as_str());
    let snapshot = store
        .media_snapshot(post.as_str())
        .await
        .expect("valid test fixture");
    TimelineEvidence::from_snapshot(&binding, &snapshot).expect("valid test fixture")
}

async fn recv(receiver: &mut tokio::sync::mpsc::UnboundedReceiver<usize>) -> usize {
    tokio::time::timeout(Duration::from_secs(1), receiver.recv())
        .await
        .expect("valid test fixture")
        .expect("valid test fixture")
}
