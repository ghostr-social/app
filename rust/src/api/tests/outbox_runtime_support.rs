//! Runtime-side outbox fixtures: an [`OutboxBootstrap`] over a plan
//! executor that records retrievals and never completes them, so a test
//! can prove nothing on the feed path waits for a relay list.

use crate::discovery::discovery_scheduler::RetrievalOutcome;
use crate::discovery::outbox_bootstrap::OutboxBootstrap;
use crate::discovery::outbox_directory::OutboxDirectory;
use crate::discovery::plan_executor::{PlanExecutor, PlanFuture, PlannedRetrieval};
use crate::discovery::relay_plan_executor::SharedOutboxDirectory;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub(crate) const BOOTSTRAP_RELAY: &str = "wss://boot.example";

struct PendingExecutor {
    started: mpsc::UnboundedSender<PlannedRetrieval>,
}

impl PlanExecutor for PendingExecutor {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture {
        let _ = self.started.send(retrieval);
        Box::pin(std::future::pending())
    }
}

pub(crate) struct BootstrapProbe {
    pub(crate) started: mpsc::UnboundedReceiver<PlannedRetrieval>,
    pub(crate) directory: SharedOutboxDirectory,
}

/// A bootstrap whose retrievals never finish, plus the probes for what
/// it asked for and what it filed. Its own outcome channel is dropped:
/// nothing here ever completes a retrieval.
pub(crate) fn test_bootstrap() -> (Arc<OutboxBootstrap>, BootstrapProbe) {
    let (started, started_probe) = mpsc::unbounded_channel();
    let (outcomes, _dropped) = mpsc::unbounded_channel::<RetrievalOutcome>();
    let directory: SharedOutboxDirectory = Arc::new(RwLock::new(OutboxDirectory::new(vec![
        BOOTSTRAP_RELAY.to_owned(),
    ])));
    let bootstrap = OutboxBootstrap::new(
        Arc::new(PendingExecutor { started }),
        directory.clone(),
        outcomes,
    );
    (
        Arc::new(bootstrap),
        BootstrapProbe {
            started: started_probe,
            directory,
        },
    )
}
