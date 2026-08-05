//! Forwards executor events while the full retrieval remains in flight.

use crate::discovery::discovery_scheduler::{
    FinishedRetrieval, RetrievalOutcome, RetrievalPurpose,
};
use crate::discovery::feed_cursor::playable_cursor;
use crate::discovery::plan_executor::{
    PlanExecutor, PlanFailure, PlanPage, PlanPageFuture, PlannedRetrieval,
};
use crate::discovery::retrieval_queue::{FeedContext, RetrievalRequest};
use crate::discovery::search_queries::QueryPlan;
use nostr_sdk::Event;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const PROGRESS_BUFFER: usize = 8;
pub(crate) const MAX_PROGRESS_OUTCOMES: usize = 64;

pub(crate) struct RetrievalTaskInput {
    pub(crate) task_id: u64,
    pub(crate) executor: Arc<dyn PlanExecutor>,
    pub(crate) finished: mpsc::UnboundedSender<FinishedRetrieval>,
    pub(crate) outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    pub(crate) request: RetrievalRequest,
    pub(crate) plan: QueryPlan,
}

pub(crate) fn spawn_retrieval_task(input: RetrievalTaskInput) -> JoinHandle<()> {
    tokio::spawn(async move {
        let context = input.request.context.clone();
        let retrieval = PlannedRetrieval {
            context: input.request.context,
            priority: input.request.priority,
            plan: input.plan,
        };
        let purpose = purpose(&retrieval);
        let progress =
            execute_progressively(input.executor, retrieval, context.clone(), input.outcomes).await;
        let _ = input.finished.send(FinishedRetrieval {
            task_id: input.task_id,
            context,
            result: progress.result,
            purpose,
            had_playable_progress: progress.had_playable,
        });
    })
}

pub(crate) fn purpose(retrieval: &PlannedRetrieval) -> RetrievalPurpose {
    if retrieval
        .plan
        .queries
        .first()
        .is_none_or(|query| query.filter.until.is_none())
    {
        RetrievalPurpose::Head
    } else {
        RetrievalPurpose::Older
    }
}

pub(crate) async fn execute_progressively(
    executor: Arc<dyn PlanExecutor>,
    retrieval: PlannedRetrieval,
    context: FeedContext,
    outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
) -> ProgressiveResult {
    let (progress, mut events) = mpsc::channel(PROGRESS_BUFFER);
    let mut execution = executor.execute_page_with_progress(retrieval, progress);
    forward_events(&mut execution, &mut events, context, outcomes).await
}

async fn forward_events(
    execution: &mut PlanPageFuture,
    events: &mut mpsc::Receiver<Event>,
    context: FeedContext,
    outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
) -> ProgressiveResult {
    let mut reported = 0;
    let mut had_playable = false;
    loop {
        tokio::select! {
            biased;
            Some(event) = events.recv() => {
                had_playable |= playable_cursor(std::slice::from_ref(&event)).is_some();
                if reported < MAX_PROGRESS_OUTCOMES {
                    send_progress(&outcomes, &context, event);
                    reported += 1;
                }
            },
            result = &mut *execution => return ProgressiveResult { result, had_playable },
        }
    }
}

pub(crate) struct ProgressiveResult {
    result: Result<PlanPage, PlanFailure>,
    had_playable: bool,
}

fn send_progress(
    outcomes: &mpsc::UnboundedSender<RetrievalOutcome>,
    context: &FeedContext,
    event: Event,
) {
    let _ = outcomes.send(RetrievalOutcome::Progress {
        context: context.clone(),
        event: Box::new(event),
    });
}
