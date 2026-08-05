//! The scheduler's event loop: waits for one command, finished
//! retrieval, or inventory-mode transition, applies it, then fills the
//! free worker slots — never a periodic wake-up.

use crate::discovery::control_loop::{discovery_action, DiscoveryAction};
use crate::discovery::discovery_scheduler::{
    ActiveRetrieval, DiscoveryCommand, FinishedRetrieval, RetrievalOutcome, RetrievalPurpose,
    SchedulerWorker,
};
use crate::discovery::feed_cursor::playable_cursor;
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority, RetrievalRequest};
use crate::discovery::scheduler_plans::widened_plan;
use crate::discovery::scheduler_progress::{spawn_retrieval_task, RetrievalTaskInput};
use crate::discovery::search_queries::{plan_discovery, QueryPlan};
use crate::engine::inventory_controller::Mode;
use nostr_sdk::{Event, Timestamp};

enum Wake {
    Command(DiscoveryCommand),
    Finished(FinishedRetrieval),
    Mode(Mode),
    Quiet,
}

impl SchedulerWorker {
    /// Waits for one event, applies it, and pumps the queue. Returns
    /// `false` only when the control channel is gone (shutdown).
    pub(crate) async fn step(&mut self) -> bool {
        let Some(wake) = self.next_wake().await else {
            return false;
        };
        self.apply(wake);
        self.pump();
        true
    }

    async fn next_wake(&mut self) -> Option<Wake> {
        tokio::select! {
            command = self.commands.recv() => command.map(Wake::Command),
            Some(done) = self.finished.recv() => Some(Wake::Finished(done)),
            changed = self.modes.changed(), if self.modes_live => Some(self.mode_wake(changed.is_ok())),
        }
    }

    /// A dead mode sender quiets the branch instead of busy-waking.
    fn mode_wake(&mut self, live: bool) -> Wake {
        self.modes_live = live;
        if live {
            Wake::Mode(*self.modes.borrow_and_update())
        } else {
            Wake::Quiet
        }
    }

    fn apply(&mut self, wake: Wake) {
        match wake {
            Wake::Command(command) => self.apply_command(command),
            Wake::Finished(done) => self.finish(done),
            Wake::Mode(mode) => self.apply_mode(mode),
            Wake::Quiet => {}
        }
    }

    /// Unified control loop (plan §5.4): hunger widens the active
    /// feed's querying, comfort keeps the radio quiet.
    fn apply_mode(&mut self, mode: Mode) {
        let Some(context) = self.feeds.active().cloned() else {
            return;
        };
        if self.feeds.is_query(&context) {
            return;
        }
        let queued = self.queue.has_pending(&context);
        match discovery_action(mode, self.feeds.query_state(&context, queued)) {
            DiscoveryAction::Idle => {}
            DiscoveryAction::PrefetchNextPage => self.prefetch(context),
            DiscoveryAction::WidenActiveQuery => self.widen(context),
        }
    }

    pub(crate) fn prefetch(&mut self, context: FeedContext) {
        let Some(request) = self.feeds.older_page_request(&context, None) else {
            return;
        };
        self.enqueue(
            context,
            RetrievalPriority::Background,
            plan_discovery(&request),
        );
    }

    fn widen(&mut self, context: FeedContext) {
        let Some(request) = self.feeds.base_request(&context).cloned() else {
            return;
        };
        self.feeds.mark_widened(&context);
        self.enqueue(
            context,
            RetrievalPriority::Background,
            widened_plan(&request),
        );
    }

    fn finish(&mut self, done: FinishedRetrieval) {
        self.tasks.remove(&done.task_id);
        self.feeds.record_done(&done.context);
        let cursor = done.result.as_ref().ok().and_then(|page| page.cursor);
        let result = done.result.map(|page| page.events);
        let result = match self.queries.finish(&done.context, result) {
            Ok(()) => return,
            Err(result) => result,
        };
        self.record_feed_result(&done.context, &result, cursor, done.purpose);
        let context = done.context;
        let _ = self.outcomes.send(RetrievalOutcome::Completed {
            context: context.clone(),
            result,
            purpose: done.purpose,
        });
        self.advance_query(context);
    }

    fn record_feed_result(
        &mut self,
        context: &FeedContext,
        result: &Result<Vec<Event>, PlanFailure>,
        query_cursor: Option<Timestamp>,
        purpose: RetrievalPurpose,
    ) {
        let Ok(events) = result else {
            return self.feeds.record_failure(context);
        };
        let cursor = if self.feeds.is_query(context) {
            query_cursor
        } else {
            playable_cursor(events)
        };
        self.feeds
            .record_page(context, cursor, purpose == RetrievalPurpose::Head);
    }

    fn pump(&mut self) {
        while self.feeds.total_inflight() < self.max_concurrent {
            let Some((request, plan)) = self.queue.take_next() else {
                return;
            };
            self.feeds.record_start(&request.context);
            self.spawn_retrieval(request, plan);
        }
    }

    fn spawn_retrieval(&mut self, request: RetrievalRequest, plan: QueryPlan) {
        let task_id = self.next_task_id;
        self.next_task_id = self.next_task_id.wrapping_add(1);
        let task_context = request.context.clone();
        let task = spawn_retrieval_task(RetrievalTaskInput {
            task_id,
            executor: self.executor.clone(),
            finished: self.finished_sender.clone(),
            outcomes: self.outcomes.clone(),
            request,
            plan,
        });
        self.tasks.insert(
            task_id,
            ActiveRetrieval {
                context: task_context,
                abort: task.abort_handle(),
            },
        );
    }
}
