use crate::api::feed_outcomes::file_lists_for;
use crate::api::feed_runtime::{lock, SharedFeedState};
use crate::discovery::discovery_scheduler::{RetrievalOutcome, RetrievalPurpose};
use crate::discovery::outbox_bootstrap::OutboxBootstrap;
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::retrieval_queue::FeedContext;
use crate::video::delivery_events::DeliveryHandle;
use flutter_rust_bridge::frb;
use nostr_sdk::Event;
use std::sync::Arc;
use tokio::sync::mpsc;

#[frb(ignore)]
pub(crate) struct OutcomeSinks {
    pub(crate) state: SharedFeedState,
    pub(crate) bootstrap: Arc<OutboxBootstrap>,
    pub(crate) candidates: Option<DeliveryHandle>,
}

pub(crate) async fn pump_outcomes(
    sinks: OutcomeSinks,
    mut outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
) {
    while let Some(outcome) = outcomes.recv().await {
        apply_outcome(&sinks, outcome).await;
    }
}

async fn apply_outcome(sinks: &OutcomeSinks, outcome: RetrievalOutcome) {
    match outcome {
        RetrievalOutcome::Started { context } => {
            lock(&sinks.state).apply_started(&context);
        }
        RetrievalOutcome::Progress { context, event } => {
            let candidate = lock(&sinks.state).apply_progress(&context, *event);
            crate::api::candidate_delivery::admit(sinks.candidates.as_ref(), candidate);
        }
        RetrievalOutcome::Completed {
            context,
            result,
            purpose,
        } => apply_completed(sinks, context, result, purpose).await,
    }
}

async fn apply_completed(
    sinks: &OutcomeSinks,
    context: FeedContext,
    result: Result<Vec<Event>, PlanFailure>,
    purpose: RetrievalPurpose,
) {
    if let Ok(events) = &result {
        file_lists_for(sinks, context.session(), events).await;
    }
    let admitted = lock(&sinks.state).apply_retrieval(&context, result, purpose);
    for candidate in admitted {
        crate::api::candidate_delivery::admit(sinks.candidates.as_ref(), Some(candidate));
    }
}
