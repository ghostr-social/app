use crate::api::feed::outcomes::file_lists_for;
use crate::api::runtime::discovery::{lock, SharedFeedState};
use crate::discovery::outbox::bootstrap::OutboxBootstrap;
use crate::discovery::retrieval_types::{
    FeedContext, PlanFailure, RetrievalOutcome, RetrievalPurpose,
};
use flutter_rust_bridge::frb;
use ghostr_delivery::delivery_events::DeliveryHandle;
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
            crate::api::delivery::candidates::admit(sinks.candidates.as_ref(), candidate);
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
        crate::api::delivery::candidates::admit(sinks.candidates.as_ref(), Some(candidate));
    }
}
