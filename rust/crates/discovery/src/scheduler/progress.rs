//! Forwards executor events while the full retrieval remains in flight.

use crate::feed::cursor::playable_cursor;
use crate::plan_executor::{PlanExecutor, PlanPage, PlanPageFuture, PlannedRetrieval};
use crate::query::search::QueryPlan;
use crate::retrieval_types::{
    FeedContext, PlanFailure, RetrievalOutcome, RetrievalPriority, RetrievalPurpose,
    RetrievalRequest,
};
use crate::scheduler::FinishedRetrieval;
use nostr_sdk::Event;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

const PROGRESS_BUFFER: usize = 8;

pub(crate) struct RetrievalTaskInput {
    pub(super) task_id: u64,
    pub(super) executor: Arc<dyn PlanExecutor>,
    pub(super) finished: mpsc::UnboundedSender<FinishedRetrieval>,
    pub(super) outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
    pub(super) request: RetrievalRequest,
    pub(super) plan: QueryPlan,
    pub(super) deferred_reposts: Vec<Event>,
}

pub(crate) fn spawn_retrieval_task(input: RetrievalTaskInput) -> JoinHandle<()> {
    tokio::spawn(async move {
        let context = input.request.context.clone();
        let retrieval = PlannedRetrieval {
            context: input.request.context,
            priority: input.request.priority,
            plan: input.plan,
            deferred_reposts: input.deferred_reposts,
        };
        let purpose = purpose(&retrieval);
        let progress =
            execute_retrieval(input.executor, retrieval, context.clone(), input.outcomes).await;
        let _ = input.finished.send(FinishedRetrieval {
            task_id: input.task_id,
            context,
            result: progress.result,
            purpose,
            had_playable_progress: progress.had_playable,
        });
    })
}

async fn execute_retrieval(
    executor: Arc<dyn PlanExecutor>,
    retrieval: PlannedRetrieval,
    context: FeedContext,
    outcomes: mpsc::UnboundedSender<RetrievalOutcome>,
) -> ProgressiveResult {
    if retrieval.priority == RetrievalPriority::Enrichment {
        return ProgressiveResult {
            result: executor.execute_page(retrieval).await,
            had_playable: false,
        };
    }
    execute_progressively(executor, retrieval, context, outcomes).await
}

fn purpose(retrieval: &PlannedRetrieval) -> RetrievalPurpose {
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

async fn execute_progressively(
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
    let mut had_playable = false;
    loop {
        tokio::select! {
            biased;
            Some(event) = events.recv() => {
                forward_event(&outcomes, &context, event, &mut had_playable);
            },
            result = &mut *execution => {
                drain_buffer(events, &outcomes, &context, &mut had_playable);
                return ProgressiveResult { result, had_playable };
            },
        }
    }
}

struct ProgressiveResult {
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

fn drain_buffer(
    events: &mut mpsc::Receiver<Event>,
    outcomes: &mpsc::UnboundedSender<RetrievalOutcome>,
    context: &FeedContext,
    had_playable: &mut bool,
) {
    while let Ok(event) = events.try_recv() {
        forward_event(outcomes, context, event, had_playable);
    }
}

fn forward_event(
    outcomes: &mpsc::UnboundedSender<RetrievalOutcome>,
    context: &FeedContext,
    event: Event,
    had_playable: &mut bool,
) {
    *had_playable |= playable_cursor(core::slice::from_ref(&event)).is_some();
    send_progress(outcomes, context, event);
}
