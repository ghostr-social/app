//! Seam between the discovery scheduler and relay IO: an executor runs
//! one planned retrieval; tests inject fakes so scheduling logic never
//! touches relays.

use crate::feed::cursor::retrieval_cursor;
use crate::query::search::QueryPlan;
use crate::retrieval_types::{EventProgress, FeedContext, PlanFailure, RetrievalPriority};
use core::future::Future;
use core::pin::Pin;
use nostr_sdk::{Event, EventId, Timestamp};

/// One retrieval as handed to the executor.
#[derive(Clone, Debug)]
pub struct PlannedRetrieval {
    pub(super) context: FeedContext,
    pub(super) priority: RetrievalPriority,
    pub plan: QueryPlan,
    pub(super) deferred_reposts: Vec<Event>,
}

/// Boxed result future so the executor trait stays object-safe.
pub type PlanFuture = Pin<Box<dyn Future<Output = Result<Vec<Event>, PlanFailure>> + Send>>;

/// A completed scheduled page plus the conservative cursor its wire filters
/// reached. Generic executors derive it from their merged events.
pub struct PlanPage {
    pub(super) events: Vec<Event>,
    pub(super) cursor: Option<Timestamp>,
    pub(super) complete: bool,
    pub(super) repost_retry: RepostRetryDelta,
}

impl PlanPage {
    fn from_events(events: Vec<Event>) -> Self {
        let cursor = retrieval_cursor(&events);
        Self {
            events,
            cursor,
            complete: true,
            repost_retry: RepostRetryDelta::default(),
        }
    }
}

#[derive(Default)]
pub(crate) struct RepostRetryDelta {
    pub(super) considered: Vec<EventId>,
    pub(super) deferred: Vec<Event>,
}

impl RepostRetryDelta {}

pub type PlanPageFuture = Pin<Box<dyn Future<Output = Result<PlanPage, PlanFailure>> + Send>>;

/// Executes one planned retrieval against relays (or a fake in tests).
pub trait PlanExecutor: Send + Sync {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture;

    fn execute_page(&self, retrieval: PlannedRetrieval) -> PlanPageFuture {
        let execution = self.execute(retrieval);
        Box::pin(async move { execution.await.map(PlanPage::from_events) })
    }

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

#[cfg(test)]
#[path = "plan_executor_axiom_test.rs"]
mod axiom_test_support;
