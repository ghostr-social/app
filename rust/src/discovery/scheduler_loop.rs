//! The scheduler's event loop: waits for one command, finished
//! retrieval, or inventory-mode transition, applies it, then fills the
//! free worker slots — never a periodic wake-up.

use crate::discovery::control_loop::{discovery_action, DiscoveryAction};
use crate::discovery::discovery_scheduler::{
    max_concurrent_requests, DiscoveryCommand, FinishedRetrieval, RetrievalOutcome,
    SchedulerWorker,
};
use crate::discovery::pagination::next_page_cursor;
use crate::discovery::plan_executor::PlannedRetrieval;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority, RetrievalRequest};
use crate::discovery::search_queries::{plan_discovery, QueryPlan};
use crate::discovery::video_filters::{DiscoveryRequest, WIDE_QUERY_LIMIT};
use crate::engine::inventory_controller::Mode;
use nostr_sdk::Timestamp;

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

    fn apply_command(&mut self, command: DiscoveryCommand) {
        match command {
            DiscoveryCommand::OpenFeed { context, request } => self.open_feed(context, request),
            DiscoveryCommand::LoadMore { context, older_than } => {
                self.load_more(context, older_than);
            }
            DiscoveryCommand::Focus(context) => self.queue.focus(context),
            DiscoveryCommand::Background { context, request } => {
                self.enqueue(context, RetrievalPriority::Background, plan_discovery(&request));
            }
            DiscoveryCommand::SetDataUsage(level) => {
                self.max_concurrent = max_concurrent_requests(level);
            }
        }
    }

    fn open_feed(&mut self, context: FeedContext, request: DiscoveryRequest) {
        self.feeds.open(context.clone(), request.clone());
        self.queue.focus(context.clone());
        self.enqueue(context, RetrievalPriority::Interactive, plan_discovery(&request));
    }

    fn load_more(&mut self, context: FeedContext, older_than: Option<Timestamp>) {
        let Some(request) = self.feeds.older_page_request(&context, older_than) else {
            return;
        };
        self.queue.focus(context.clone());
        self.enqueue(context, RetrievalPriority::Interactive, plan_discovery(&request));
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
        self.enqueue(context, RetrievalPriority::Background, plan_discovery(&request));
    }

    fn widen(&mut self, context: FeedContext) {
        let Some(request) = self.feeds.base_request(&context).cloned() else {
            return;
        };
        self.feeds.mark_widened(&context);
        self.enqueue(context, RetrievalPriority::Background, widened_plan(&request));
    }

    fn finish(&mut self, done: FinishedRetrieval) {
        self.feeds.record_done(&done.context);
        if let Ok(events) = &done.result {
            let cursor = next_page_cursor(events.iter().map(|event| event.created_at));
            self.feeds.record_page(&done.context, cursor);
        }
        let _ = self.outcomes.send(RetrievalOutcome {
            context: done.context,
            result: done.result,
        });
    }

    fn enqueue(&mut self, context: FeedContext, priority: RetrievalPriority, plan: QueryPlan) {
        self.queue.push(RetrievalRequest { context, priority }, plan);
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

    fn spawn_retrieval(&self, request: RetrievalRequest, plan: QueryPlan) {
        let executor = self.executor.clone();
        let finished = self.finished_sender.clone();
        tokio::spawn(async move {
            let context = request.context.clone();
            let retrieval = PlannedRetrieval {
                context: request.context,
                priority: request.priority,
                plan,
            };
            let result = executor.execute(retrieval).await;
            let _ = finished.send(FinishedRetrieval { context, result });
        });
    }
}

/// Hunger widening: the primary video query re-issued at the wide
/// limit (`video_filters::WIDE_QUERY_LIMIT`) to deepen the pool.
fn widened_plan(request: &DiscoveryRequest) -> QueryPlan {
    let mut plan = plan_discovery(request);
    if let Some(primary) = plan.queries.first_mut() {
        primary.filter = primary.filter.clone().limit(WIDE_QUERY_LIMIT);
    }
    plan
}
