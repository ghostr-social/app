//! Seam between the discovery scheduler and relay IO: an executor runs
//! one planned retrieval; tests inject fakes so scheduling logic never
//! touches relays.

use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::discovery::search_queries::QueryPlan;
use nostr_sdk::Event;
use std::future::Future;
use std::pin::Pin;

/// One retrieval as handed to the executor.
#[derive(Clone, Debug)]
pub struct PlannedRetrieval {
    pub context: FeedContext,
    pub priority: RetrievalPriority,
    pub plan: QueryPlan,
}

/// Why a whole retrieval failed. Mirrors the Dart contract in
/// ndk_nostr_video_event_query.dart: only the primary query's failure
/// sinks a load; additive hiccups never surface.
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

/// Executes one planned retrieval against relays (or a fake in tests).
pub trait PlanExecutor: Send + Sync {
    fn execute(&self, retrieval: PlannedRetrieval) -> PlanFuture;
}
