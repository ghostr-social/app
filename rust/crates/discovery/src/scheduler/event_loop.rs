//! The scheduler's event loop: waits for one command, finished
//! retrieval, or candidate-demand transition, applies it, then fills the
//! free worker slots — never a periodic wake-up.

use crate::feed::cursor::playable_cursor;
use crate::plan_executor::PlanPage;
use crate::query::search::{plan_discovery, QueryPlan};
use crate::retrieval_types::{
    FeedContext, RetrievalOutcome, RetrievalPriority, RetrievalPurpose, RetrievalRequest,
};
use crate::scheduler::control::{discovery_action, DiscoveryAction};
use crate::scheduler::plans::widened_plan;
use crate::scheduler::progress::{spawn_retrieval_task, RetrievalTaskInput};
use crate::scheduler::retry::should_retry_feed;
use crate::scheduler::{ActiveRetrieval, DiscoveryCommand, FinishedRetrieval, SchedulerWorker};
use ghostr_engine::adaptive::DiscoveryDemand;

enum Wake {
    Command(DiscoveryCommand),
    Finished(FinishedRetrieval),
    Demand(DiscoveryDemand),
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
            changed = self.demand.changed(), if self.demand_live => Some(self.demand_wake(changed.is_ok())),
        }
    }

    /// A dead demand sender quiets the branch instead of busy-waking.
    fn demand_wake(&mut self, live: bool) -> Wake {
        self.demand_live = live;
        if live {
            Wake::Demand(*self.demand.borrow_and_update())
        } else {
            Wake::Quiet
        }
    }

    fn apply(&mut self, wake: Wake) {
        match wake {
            Wake::Command(command) => self.apply_command(command),
            Wake::Finished(done) => self.finish(done),
            Wake::Demand(demand) => self.apply_demand(demand),
            Wake::Quiet => {}
        }
    }

    fn apply_demand(&mut self, demand: DiscoveryDemand) {
        let Some(context) = self.feeds.active().cloned() else {
            return;
        };
        if self.feeds.is_continuous(&context) {
            return;
        }
        let queued = self.queue.has_pending(&context);
        match discovery_action(demand, self.feeds.query_state(&context, queued)) {
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

    fn finish(&mut self, mut done: FinishedRetrieval) {
        if self.tasks.remove(&done.task_id).is_none() {
            return;
        }
        self.feeds.record_done(&done.context);
        let repost_retry = self.apply_repost_retry(&mut done);
        let page = match self.queries.finish(&done.context, done.result) {
            Ok(()) => return,
            Err(result) => result,
        };
        let cursor = page.as_ref().ok().and_then(|result| result.cursor);
        let complete = page.as_ref().is_ok_and(|result| result.complete);
        self.record_feed_result(&done.context, &page, done.purpose);
        let result = page.map(|result| result.events);
        if done.had_playable_progress {
            self.feeds.record_playable(&done.context);
        }
        let retry = repost_retry
            || should_retry_feed(
                &result,
                done.purpose,
                self.feeds.has_playable(&done.context),
                complete,
            );
        let context = done.context;
        let _ = self.outcomes.send(RetrievalOutcome::Completed {
            context: context.clone(),
            result,
            cursor,
            complete,
            purpose: done.purpose,
        });
        self.advance_feed(context, retry, done.purpose);
    }

    fn apply_repost_retry(&mut self, done: &mut FinishedRetrieval) -> bool {
        if let Ok(page) = &mut done.result {
            let delta = std::mem::take(&mut page.repost_retry);
            self.deferred_reposts.apply(&done.context, delta)
        } else {
            self.deferred_reposts.has_pending(&done.context)
        }
    }

    fn record_feed_result(
        &mut self,
        context: &FeedContext,
        result: &Result<PlanPage, crate::retrieval_types::PlanFailure>,
        purpose: RetrievalPurpose,
    ) {
        let Ok(page) = result else {
            return self.feeds.record_failure(context);
        };
        if !page.complete {
            return self.feeds.record_failure(context);
        }
        self.feeds.record_page(
            context,
            page.cursor,
            purpose == RetrievalPurpose::Head,
            playable_cursor(&page.events).is_some(),
        );
    }

    fn pump(&mut self) {
        while self.feeds.total_inflight() < self.max_concurrent {
            let blocked = self.tasks.values().map(|task| &task.context);
            let Some((request, plan)) = self.queue.take_next_excluding(blocked) else {
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
        let deferred_reposts = self.deferred_reposts.batch(&task_context);
        let task = spawn_retrieval_task(RetrievalTaskInput {
            task_id,
            executor: self.executor.clone(),
            finished: self.finished_sender.clone(),
            outcomes: self.outcomes.clone(),
            request,
            plan,
            deferred_reposts,
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
