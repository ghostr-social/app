//! The scheduler's event loop: waits for one command, finished
//! retrieval, or inventory-mode transition, applies it, then fills the
//! free worker slots — never a periodic wake-up.

use crate::discovery::control_loop::{discovery_action, DiscoveryAction};
use crate::discovery::discovery_scheduler::{
    DiscoveryCommand, FinishedRetrieval, RetrievalOutcome, SchedulerWorker,
};
use crate::discovery::feed_cursor::playable_cursor;
use crate::discovery::plan_executor::PlannedRetrieval;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority, RetrievalRequest};
use crate::discovery::scheduler_plans::widened_plan;
use crate::discovery::search_queries::{plan_discovery, QueryPlan};
use crate::engine::inventory_controller::Mode;

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
        let queued = self.queue.has_pending(&context);
        match discovery_action(mode, self.feeds.query_state(&context, queued)) {
            DiscoveryAction::Idle => {}
            DiscoveryAction::PrefetchNextPage => self.prefetch(context),
            DiscoveryAction::WidenActiveQuery => self.widen(context),
        }
    }

    fn prefetch(&mut self, context: FeedContext) {
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
        let result = match self.queries.finish(&done.context, done.result) {
            Ok(()) => return,
            Err(result) => result,
        };
        if let Ok(events) = &result {
            let cursor = playable_cursor(events);
            self.feeds.record_page(&done.context, cursor);
        }
        let _ = self.outcomes.send(RetrievalOutcome {
            context: done.context,
            result,
        });
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
        let executor = self.executor.clone();
        let finished = self.finished_sender.clone();
        let task = tokio::spawn(async move {
            let context = request.context.clone();
            let retrieval = PlannedRetrieval {
                context: request.context,
                priority: request.priority,
                plan,
            };
            let result = executor.execute(retrieval).await;
            let _ = finished.send(FinishedRetrieval {
                task_id,
                context,
                result,
            });
        });
        self.tasks.insert(task_id, task.abort_handle());
    }
}
