//! Calls made through the cloneable discovery scheduler handle.

use super::{ControlCommand, DiscoveryCommand, DiscoveryHandle, FeedCommand, WorkCommand};
use crate::query::search::QueryPlan;
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, PlanFailure};
#[cfg(test)]
use crate::scheduler::hunt::HuntToken;
use crate::scheduler::queries::QueryResult;
use crate::session_generation::SessionGeneration;
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Timestamp;
use std::sync::atomic::Ordering;
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

    #[allow(
        dead_code,
        reason = "focus control is exercised only by scheduler tests"
    )]
    pub(crate) fn focus(&self, context: FeedContext) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Feed(FeedCommand::Focus(context)));
    }

    pub fn close_feed(&self, context: FeedContext) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Feed(FeedCommand::Close(context)));
    }

    #[allow(
        dead_code,
        reason = "background control is exercised only by scheduler tests"
    )]
    pub(crate) fn background(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Work(WorkCommand::Background {
                context,
                request,
            }));
    }

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
            .map_err(|_| stopped())?;
        result.await.map_err(|_| cancelled())?
    }

    pub async fn reset_session(&self) -> Result<(), PlanFailure> {
        let (reply, result) = oneshot::channel();
        self.sender
            .send(DiscoveryCommand::Control(ControlCommand::ResetSession {
                reply,
            }))
            .map_err(|_| stopped())?;
        result.await.map_err(|_| stopped())
    }

    pub fn set_data_usage(&self, level: DataUsageLevel) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Control(ControlCommand::SetDataUsage(
                level,
            )));
    }

    #[cfg(test)]
    pub(crate) fn inject_retry(&self, context: FeedContext, token: u64) {
        let _ = self.sender.send(DiscoveryCommand::Work(WorkCommand::Retry {
            context,
            token: HuntToken(token),
        }));
    }
}

fn stopped() -> PlanFailure {
    PlanFailure::new("the discovery scheduler stopped")
}

fn cancelled() -> PlanFailure {
    PlanFailure::new("the discovery query was cancelled")
}
