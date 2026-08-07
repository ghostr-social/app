//! Calls made through the cloneable discovery scheduler handle.

use super::{DiscoveryCommand, DiscoveryHandle};
use crate::plan_executor::PlanFailure;
use crate::retrieval_queue::FeedContext;
#[cfg(test)]
use crate::scheduler_hunt::HuntToken;
use crate::scheduler_queries::QueryResult;
use crate::search_queries::QueryPlan;
use crate::session_generation::SessionGeneration;
use crate::video_filters::DiscoveryRequest;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Timestamp;
use std::sync::atomic::Ordering;
use tokio::sync::oneshot;

impl DiscoveryHandle {
    pub fn open_feed(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self
            .sender
            .send(DiscoveryCommand::OpenFeed { context, request });
    }

    pub fn load_more(&self, context: FeedContext, older_than: Option<Timestamp>) {
        let _ = self.sender.send(DiscoveryCommand::LoadMore {
            context,
            older_than,
        });
    }

    pub fn focus(&self, context: FeedContext) {
        let _ = self.sender.send(DiscoveryCommand::Focus(context));
    }

    pub fn close_feed(&self, context: FeedContext) {
        let _ = self.sender.send(DiscoveryCommand::CloseFeed(context));
    }

    pub fn background(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Background { context, request });
    }

    pub async fn query(&self, session: SessionGeneration, plan: QueryPlan) -> QueryResult {
        let sequence = self.query_sequence.fetch_add(1, Ordering::Relaxed);
        let context = FeedContext::for_session(format!("query-{sequence}"), session);
        let (reply, result) = oneshot::channel();
        self.sender
            .send(DiscoveryCommand::Query {
                context,
                plan,
                reply,
            })
            .map_err(|_| stopped())?;
        result.await.map_err(|_| cancelled())?
    }

    pub async fn reset_session(&self) -> Result<(), PlanFailure> {
        let (reply, result) = oneshot::channel();
        self.sender
            .send(DiscoveryCommand::ResetSession { reply })
            .map_err(|_| stopped())?;
        result.await.map_err(|_| stopped())
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        let _ = self.sender.send(DiscoveryCommand::SetDataUsage(level));
    }

    #[cfg(test)]
    pub(crate) fn inject_retry(&self, context: FeedContext, token: u64) {
        let _ = self.sender.send(DiscoveryCommand::RetryFeed {
            context,
            token: HuntToken(token),
        });
    }
}

fn stopped() -> PlanFailure {
    PlanFailure::new("the discovery scheduler stopped")
}

fn cancelled() -> PlanFailure {
    PlanFailure::new("the discovery query was cancelled")
}
