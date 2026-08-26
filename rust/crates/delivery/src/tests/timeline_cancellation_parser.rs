use crate::manager::timeline::axiom_test_support::TimelineIncomplete;
use crate::manager::timeline::axiom_test_support::TimelineInput;
use crate::manager::timeline::axiom_test_support::TimelineParse;
use crate::manager::timeline::axiom_test_support::TimelineParser;
use crate::manager::timeline::{TimelineCoordinator, TimelineEvidence, TimelineResult, TimelineTerminal};
use crate::tests::demand_lease_fixture::{binding, catalog};
use crate::tests::support::temp_directory;
use ghostr_engine::media_timeline::TimelineParseControl;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

pub(crate) struct CancellationHarness {
    pub(super) root: PathBuf,
    pub(super) coordinator: TimelineCoordinator,
    pub(super) evidence: TimelineEvidence,
    pub(super) post: PostId,
    started: mpsc::UnboundedReceiver<usize>,
}

impl CancellationHarness {
    pub(super) async fn new() -> Self {
        let root = temp_directory("timeline-cancellation");
        let store = Arc::new(PartialRangeStore::with_capacity(
            root.clone(),
            Arc::new(Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        ));
        let catalog = catalog(&["post"]);
        let binding = binding(&catalog, "post");
        store.bind_representation(binding.clone()).await.expect("valid test fixture");
        store.set_total_len("post", 32).await.expect("valid test fixture");
        store.write_range("post", 0, b"abcdefgh").await.expect("valid test fixture");
        let snapshot = store.media_snapshot("post").await.expect("valid test fixture");
        let evidence = TimelineEvidence::from_snapshot(&binding, &snapshot).expect("valid test fixture");
        let (parser, started) = CancellationParser::new();
        let coordinator = TimelineCoordinator::with_parser(store, Arc::new(parser), 1);
        Self {
            root,
            coordinator,
            evidence,
            post: PostId::new("post"),
            started,
        }
    }

    pub(super) async fn next_started(&mut self) -> usize {
        tokio::time::timeout(core::time::Duration::from_secs(1), self.started.recv())
            .await
            .expect("valid test fixture")
            .expect("valid test fixture")
    }

    pub(super) async fn next_result(&mut self) -> TimelineResult {
        tokio::time::timeout(core::time::Duration::from_secs(1), self.coordinator.recv())
            .await
            .expect("valid test fixture")
            .expect("valid test fixture")
    }
}

pub(crate) struct CancellationParser {
    calls: AtomicUsize,
    started: mpsc::UnboundedSender<usize>,
}

impl CancellationParser {
    fn new() -> (Self, mpsc::UnboundedReceiver<usize>) {
        let (started, receiver) = mpsc::unbounded_channel();
        (
            Self {
                calls: AtomicUsize::new(0),
                started,
            },
            receiver,
        )
    }
}

impl TimelineParser for CancellationParser {
    fn parse(&self, _input: TimelineInput, control: &dyn TimelineParseControl) -> TimelineParse {
        let call = self.calls.fetch_add(1, Ordering::AcqRel);
        self.started.send(call).expect("valid test fixture");
        if call > 0 {
            return TimelineParse::Completed(TimelineTerminal::Incomplete(
                TimelineIncomplete::Unavailable,
            ));
        }
        while !control.is_cancelled() {
            std::thread::yield_now();
        }
        TimelineParse::Cancelled
    }
}
