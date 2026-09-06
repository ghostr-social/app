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

mod progress;

const OUTCOMES_PER_TURN: usize = 32;
type ReadyOutcomes = core::iter::Peekable<std::vec::IntoIter<RetrievalOutcome>>;

#[frb(ignore)]
pub(crate) struct OutcomeSinks {
    pub(crate) state: SharedFeedState,
    pub(crate) bootstrap: Arc<OutboxBootstrap>,
    pub(crate) candidates: Option<DeliveryHandle>,
}

struct CompletedOutcome {
    context: FeedContext,
    result: Result<Vec<Event>, PlanFailure>,
    cursor: Option<nostr_sdk::Timestamp>,
    complete: bool,
    purpose: RetrievalPurpose,
}

pub(crate) async fn pump_outcomes(
    sinks: OutcomeSinks,
    mut outcomes: mpsc::UnboundedReceiver<RetrievalOutcome>,
) {
    loop {
        let mut ready = Vec::with_capacity(OUTCOMES_PER_TURN);
        if outcomes.recv_many(&mut ready, OUTCOMES_PER_TURN).await == 0 {
            break;
        }
        apply_ready(&sinks, ready).await;
        tokio::task::yield_now().await;
    }
}

async fn apply_ready(sinks: &OutcomeSinks, ready: Vec<RetrievalOutcome>) {
    let mut ready = ready.into_iter().peekable();
    while let Some(outcome) = ready.next() {
        apply_outcome(sinks, outcome, &mut ready).await;
    }
}

async fn apply_outcome(sinks: &OutcomeSinks, outcome: RetrievalOutcome, ready: &mut ReadyOutcomes) {
    match outcome {
        RetrievalOutcome::Started { context } => {
            lock(&sinks.state).apply_started(&context);
        }
        RetrievalOutcome::Progress { context, event } => {
            progress::apply(sinks, &context, *event, ready);
        }
        RetrievalOutcome::Completed {
            context,
            result,
            cursor,
            complete,
            purpose,
        } => {
            apply_completed(
                sinks,
                CompletedOutcome {
                    context,
                    result,
                    cursor,
                    complete,
                    purpose,
                },
            )
            .await;
        }
    }
}

async fn apply_completed(sinks: &OutcomeSinks, completed: CompletedOutcome) {
    if let Ok(events) = &completed.result {
        file_lists_for(sinks, completed.context.session(), events).await;
    }
    let admitted = lock(&sinks.state).apply_retrieval(
        &completed.context,
        completed.result,
        completed.cursor,
        completed.purpose,
        completed.complete,
    );
    for candidate in admitted {
        crate::api::delivery::candidates::admit(sinks.candidates.as_ref(), Some(candidate));
    }
}
