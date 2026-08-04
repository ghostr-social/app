//! Populates the outbox directory and the social graph in the
//! background (plan §5.2). Relay lists are an optimisation: a feed must
//! never wait for one, so every retrieval here runs on its own task
//! instead of taking a worker slot in the discovery scheduler's pool,
//! and its results flow through the same outcome channel a feed page
//! uses. The first page therefore leaves with whatever the directory
//! already knows, and every later page benefits from what landed since.

use crate::discovery::discovery_scheduler::RetrievalOutcome;
use crate::discovery::outbox_plans::{
    author_relay_lists_plan, viewer_lists_plan, MAX_RELAY_LIST_AUTHORS,
};
use crate::discovery::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::discovery::relay_plan_executor::SharedOutboxDirectory;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::discovery::search_queries::QueryPlan;
use nostr_sdk::{Event, Kind, PublicKey};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Context every relay-list retrieval reports under. No feed answers to
/// it, so its events only ever reach the shared directory and graph.
pub const OUTBOX_CONTEXT: &str = "outbox";

/// Chases the NIP-65/NIP-02 lists a routed feed needs and remembers who
/// was already asked for, so reopening a feed costs nothing.
pub struct OutboxBootstrap {
    executor: Arc<dyn PlanExecutor>,
    outbox: SharedOutboxDirectory,
    outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    requested: Requested,
}

/// Pubkeys already asked for, shared with the retrieval tasks so a
/// failure can hand its claim back.
type Requested = Arc<Mutex<HashSet<PublicKey>>>;

impl OutboxBootstrap {
    pub fn new(
        executor: Arc<dyn PlanExecutor>,
        outbox: SharedOutboxDirectory,
        outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    ) -> Self {
        Self {
            executor,
            outbox,
            outcomes,
            requested: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Chases the viewer's own follow, mute, and relay lists once.
    pub fn viewer(&self, viewer: PublicKey) {
        let claimed = self.claim(&[viewer]);
        if claimed.is_empty() {
            return;
        }
        self.spawn(claimed, viewer_lists_plan(viewer));
    }

    /// Chases the relay lists of authors a feed just opened on (profile
    /// creators, or the viewer's follows).
    pub fn authors(&self, authors: &[PublicKey]) {
        for batch in self.claim(authors).chunks(MAX_RELAY_LIST_AUTHORS) {
            self.spawn(batch.to_vec(), author_relay_lists_plan(batch));
        }
    }

    /// Adopts a landed follow set as the main feed's routing set and
    /// chases the relay lists it does not know yet.
    pub async fn track_follows(&self, follows: Vec<PublicKey>) {
        self.outbox.write().await.track_viewer_follows(follows.clone());
        self.authors(&follows);
    }

    /// Files every relay list in a retrieval's events. Every page flows
    /// through here, so a page carrying none never takes the lock.
    pub async fn ingest(&self, events: &[Event]) {
        if !events.iter().any(|event| event.kind == Kind::RelayList) {
            return;
        }
        self.outbox.write().await.ingest_all(events);
    }

    /// The pubkeys not asked for before, now marked as asked for.
    fn claim(&self, authors: &[PublicKey]) -> Vec<PublicKey> {
        let mut requested = locked(&self.requested);
        authors
            .iter()
            .filter(|author| requested.insert(**author))
            .copied()
            .collect()
    }

    fn spawn(&self, claimed: Vec<PublicKey>, plan: QueryPlan) {
        let retrieval = PlannedRetrieval {
            context: FeedContext::new(OUTBOX_CONTEXT),
            priority: RetrievalPriority::Background,
            plan,
        };
        let executor = self.executor.clone();
        let outcomes = self.outcomes.clone();
        let requested = self.requested.clone();
        tokio::spawn(async move {
            let result = executor.execute(retrieval).await;
            if result.is_err() {
                release(&requested, &claimed);
            }
            let _ = outcomes.send(RetrievalOutcome {
                context: FeedContext::new(OUTBOX_CONTEXT),
                result,
            });
        });
    }
}

/// Unreachable relays must not cost the session its NIP-65 routing for
/// good: a failed chase hands its claim back, so the next feed open
/// asks again.
fn release(requested: &Requested, claimed: &[PublicKey]) {
    let mut requested = locked(requested);
    for author in claimed {
        requested.remove(author);
    }
}

fn locked(requested: &Requested) -> std::sync::MutexGuard<'_, HashSet<PublicKey>> {
    requested
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
