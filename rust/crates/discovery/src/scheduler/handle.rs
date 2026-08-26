//! Calls made through the cloneable discovery scheduler handle.

use super::{ControlCommand, DiscoveryCommand, DiscoveryHandle, FeedCommand, WorkCommand};
use crate::query::search::QueryPlan;
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, PlanFailure};

use crate::scheduler::queries::QueryResult;
use crate::session_generation::SessionGeneration;
use core::sync::atomic::Ordering;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Timestamp;
use tokio::sync::oneshot;

impl DiscoveryHandle {
    pub fn open_feed(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self.sender.send(DiscoveryCommand::Feed(FeedCommand::Open {
            context,
            request,
        }));
    }

    pub fn load_more(&self, context: FeedContext, older_than: Option<Timestamp>) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Feed(FeedCommand::LoadMore {
                context,
                older_than,
            }));
    }

    pub fn close_feed(&self, context: FeedContext) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Feed(FeedCommand::Close(context)));
    }

    /// Executes one bounded query against the active discovery session.
    ///
    /// # Errors
    ///
    /// Returns a plan failure when the scheduler stops, cancels, or rejects the query.
    pub async fn query(&self, session: SessionGeneration, plan: QueryPlan) -> QueryResult {
        let sequence = self.query_sequence.fetch_add(1, Ordering::Relaxed);
        let context = FeedContext::for_session(format!("query-{sequence}"), session);
        let (reply, result) = oneshot::channel();
        self.sender
            .send(DiscoveryCommand::Work(WorkCommand::Query {
                context,
                plan,
                reply,
            }))
            .map_err(|error| {
                log::warn!("Could not submit discovery query: {error}");
                stopped()
            })?;
        result.await.map_err(|error| {
            log::warn!("Discovery query reply was cancelled: {error}");
            cancelled()
        })?
    }

    /// Clears scheduler state after an account-session transition.
    ///
    /// # Errors
    ///
    /// Returns a plan failure when the scheduler cannot accept or acknowledge the reset.
    pub async fn reset_session(&self) -> Result<(), PlanFailure> {
        let (reply, result) = oneshot::channel();
        self.sender
            .send(DiscoveryCommand::Control(ControlCommand::ResetSession {
                reply,
            }))
            .map_err(|error| {
                log::warn!("Could not submit discovery session reset: {error}");
                stopped()
            })?;
        result.await.map_err(|error| {
            log::warn!("Discovery session reset acknowledgement failed: {error}");
            stopped()
        })
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Control(ControlCommand::SetDataUsage(
                level,
            )));
    }
}

fn stopped() -> PlanFailure {
    PlanFailure::new("the discovery scheduler stopped")
}

fn cancelled() -> PlanFailure {
    PlanFailure::new("the discovery query was cancelled")
}

#[cfg(test)]
#[path = "handle_axiom_test.rs"]
pub(crate) mod axiom_test_support;
