//! Seam between the discovery scheduler and relay IO: an executor runs
//! one planned retrieval; tests inject fakes so scheduling logic never
//! touches relays.

use crate::discovery::feed_cursor::retrieval_cursor;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::discovery::search_queries::QueryPlan;
use nostr_sdk::{Event, Timestamp};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;

/// One retrieval as handed to the executor.
#[derive(Clone, Debug)]
pub struct PlannedRetrieval {
    pub context: FeedContext,
    pub priority: RetrievalPriority,
    pub plan: QueryPlan,
}

/// Why a whole retrieval could not settle. Any planned content-query failure
/// leaves the page retryable so pagination never commits a partial boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanFailure {
    pub message: String,
}

impl PlanFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Boxed result future so the executor trait stays object-safe.
pub type PlanFuture = Pin<Box<dyn Future<Output = Result<Vec<Event>, PlanFailure>> + Send>>;

/// A completed scheduled page plus the conservative cursor its wire filters
/// reached. Generic executors derive it from their merged events.
pub struct PlanPage {
    pub events: Vec<Event>,
    pub cursor: Option<Timestamp>,
}

impl PlanPage {
    fn from_events(events: Vec<Event>) -> Self {
        let cursor = retrieval_cursor(&events);
        Self { events, cursor }
    }
}

pub type PlanPageFuture = Pin<Box<dyn Future<Output = Result<PlanPage, PlanFailure>> + Send>>;

/// Events yielded before a retrieval reaches EOSE/timeout and profile
/// enrichment. The scheduler turns them into provisional feed updates.
pub type EventProgress = mpsc::Sender<Event>;

/// Executes one planned retrieval against relays (or a fake in tests).
pub trait PlanExecutor: Send + Sync {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture;

    fn execute_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        _progress: EventProgress,
    ) -> PlanFuture {
        self.execute(retrieval)
    }

    fn execute_page_with_progress(
        &self,
        retrieval: PlannedRetrieval,
        progress: EventProgress,
    ) -> PlanPageFuture {
        let execution = self.execute_with_progress(retrieval, progress);
        Box::pin(async move { execution.await.map(PlanPage::from_events) })
    }
}
