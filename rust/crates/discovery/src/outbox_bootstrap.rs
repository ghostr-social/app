//! Populates the outbox directory and the social graph in the
//! background (plan §5.2). Relay lists are an optimisation: a feed must
//! never wait for one, so every retrieval here runs on its own task
//! instead of taking a worker slot in the discovery scheduler's pool,
//! and its results flow through the same outcome channel a feed page
//! uses. The first page therefore leaves with whatever the directory
//! already knows, and every later page benefits from what landed since.

use crate::discovery_scheduler::RetrievalOutcome;
use crate::outbox_plans::{
    author_relay_lists_plan, viewer_lists_plan, MAX_RELAY_LIST_AUTHORS,
};
use crate::plan_executor::{PlanExecutor, PlannedRetrieval};
use crate::relay_plan_executor::SharedOutboxDirectory;
use crate::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::search_queries::QueryPlan;
use crate::session_generation::SessionGeneration;
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
    session: SharedSession,
}

type SharedSession = Arc<Mutex<BootstrapSession>>;

struct BootstrapSession {
    generation: SessionGeneration,
    requested: HashSet<PublicKey>,
}

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
            session: Arc::new(Mutex::new(BootstrapSession {
                generation: SessionGeneration::initial(),
                requested: HashSet::new(),
            })),
        }
    }

    pub fn reset_session(&self, generation: SessionGeneration) {
        let mut session = locked(&self.session);
        session.generation = generation;
        session.requested.clear();
    }

    /// Chases the viewer's own follow, mute, and relay lists once.
    pub fn viewer(&self, viewer: PublicKey) {
        let (generation, claimed) = self.claim(&[viewer]);
        if claimed.is_empty() {
            return;
        }
        self.spawn(generation, claimed, viewer_lists_plan(viewer));
    }

    /// Chases the relay lists of authors a feed just opened on (profile
    /// creators, or the viewer's follows).
    pub fn authors(&self, authors: &[PublicKey]) {
        let generation = locked(&self.session).generation;
        self.authors_for(generation, authors);
    }

    /// Adopts a landed follow set as the main feed's routing set and
    /// chases the relay lists it does not know yet.
    pub async fn track_follows(&self, follows: Vec<PublicKey>) {
        let generation = locked(&self.session).generation;
        self.track_follows_for(generation, follows).await;
    }

    pub async fn track_follows_for(
        &self,
        generation: SessionGeneration,
        follows: Vec<PublicKey>,
    ) {
        self.outbox
            .write()
            .await
            .track_viewer_follows_for(generation, follows.clone());
        self.authors_for(generation, &follows);
    }

    /// Files every relay list in a retrieval's events. Every page flows
    /// through here, so a page carrying none never takes the lock.
    pub async fn ingest(&self, events: &[Event]) {
        let generation = locked(&self.session).generation;
        self.ingest_for(generation, events).await;
    }

    pub async fn ingest_for(&self, generation: SessionGeneration, events: &[Event]) {
        if !events.iter().any(|event| event.kind == Kind::RelayList) {
            return;
        }
        self.outbox.write().await.ingest_all_for(generation, events);
    }

    /// The pubkeys not asked for before, now marked as asked for.
    fn claim(&self, authors: &[PublicKey]) -> (SessionGeneration, Vec<PublicKey>) {
        let mut session = locked(&self.session);
        let claimed = authors
            .iter()
            .filter(|author| session.requested.insert(**author))
            .copied()
            .collect();
        (session.generation, claimed)
    }

    fn authors_for(&self, generation: SessionGeneration, authors: &[PublicKey]) {
        let claimed = self.claim_for(generation, authors);
        for batch in claimed.chunks(MAX_RELAY_LIST_AUTHORS) {
            self.spawn(generation, batch.to_vec(), author_relay_lists_plan(batch));
        }
    }

    fn claim_for(&self, generation: SessionGeneration, authors: &[PublicKey]) -> Vec<PublicKey> {
        let mut session = locked(&self.session);
        if session.generation != generation {
            return Vec::new();
        }
        authors
            .iter()
            .filter(|author| session.requested.insert(**author))
            .copied()
            .collect()
    }

    fn spawn(&self, generation: SessionGeneration, claimed: Vec<PublicKey>, plan: QueryPlan) {
        let context = FeedContext::for_session(OUTBOX_CONTEXT, generation);
        let retrieval = PlannedRetrieval {
            context: context.clone(),
            priority: RetrievalPriority::Background,
            plan,
        };
        let executor = self.executor.clone();
        let outcomes = self.outcomes.clone();
        let session = self.session.clone();
        tokio::spawn(async move {
            let result = executor.execute(retrieval).await;
            if result.is_err() {
                release(&session, generation, &claimed);
            }
            let _ = outcomes.send(RetrievalOutcome::Completed {
                context,
                result,
                purpose: crate::discovery_scheduler::RetrievalPurpose::Head,
            });
        });
    }
}

/// Unreachable relays must not cost the session its NIP-65 routing for
/// good: a failed chase hands its claim back, so the next feed open
/// asks again.
fn release(session: &SharedSession, generation: SessionGeneration, claimed: &[PublicKey]) {
    let mut session = locked(session);
    if session.generation != generation {
        return;
    }
    for author in claimed {
        session.requested.remove(author);
    }
}

fn locked(session: &SharedSession) -> std::sync::MutexGuard<'_, BootstrapSession> {
    session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
