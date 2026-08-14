//! Discovery command dispatch, kept separate from the scheduler wake loop.

use crate::query::search::{plan_discovery, QueryPlan};
use crate::query::video_filters::DiscoveryRequest;
use crate::retrieval_types::{FeedContext, RetrievalOutcome, RetrievalPriority, RetrievalRequest};
use crate::scheduler::hunt::HuntToken;
use crate::scheduler::queries::QueryResult;
use crate::scheduler::{max_concurrent_requests, SchedulerWorker};
use ghostr_engine::DataUsageLevel;
use nostr_sdk::Timestamp;
use tokio::sync::oneshot;

/// Typed command families keep scheduler dispatch bounded and explicit.
#[derive(Debug)]
pub(crate) enum DiscoveryCommand {
    Feed(FeedCommand),
    Work(WorkCommand),
    Control(ControlCommand),
}

#[derive(Debug)]
pub(crate) enum FeedCommand {
    Open {
        context: FeedContext,
        request: DiscoveryRequest,
    },
    LoadMore {
        context: FeedContext,
        older_than: Option<Timestamp>,
    },
    #[allow(
        dead_code,
        reason = "focus commands are exercised only by scheduler tests"
    )]
    Focus(FeedContext),
    Close(FeedContext),
}

#[derive(Debug)]
pub(crate) enum WorkCommand {
    Continue {
        context: FeedContext,
        token: HuntToken,
    },
    Retry {
        context: FeedContext,
        token: HuntToken,
    },
    #[allow(
        dead_code,
        reason = "background commands are exercised only by scheduler tests"
    )]
    Background {
        context: FeedContext,
        request: DiscoveryRequest,
    },
    Query {
        context: FeedContext,
        plan: QueryPlan,
        reply: oneshot::Sender<QueryResult>,
    },
}

#[derive(Debug)]
pub(crate) enum ControlCommand {
    SetDataUsage(DataUsageLevel),
    ResetSession { reply: oneshot::Sender<()> },
}

impl SchedulerWorker {
    pub(crate) fn apply_command(&mut self, command: DiscoveryCommand) {
        match command {
            DiscoveryCommand::Feed(command) => self.apply_feed_command(command),
            DiscoveryCommand::Work(command) => self.apply_work_command(command),
            DiscoveryCommand::Control(command) => self.apply_control_command(command),
        }
    }

    fn apply_feed_command(&mut self, command: FeedCommand) {
        match command {
            FeedCommand::Open { context, request } => self.open_feed(context, request),
            FeedCommand::LoadMore {
                context,
                older_than,
            } => self.load_more(context, older_than),
            FeedCommand::Focus(context) => self.queue.focus(context),
            FeedCommand::Close(context) => self.close_feed(context),
        }
    }

    fn apply_work_command(&mut self, command: WorkCommand) {
        match command {
            WorkCommand::Continue { context, token } => self.continue_feed(context, token),
            WorkCommand::Retry { context, token } => self.continue_feed_retry(context, token),
            WorkCommand::Background { context, request } => {
                self.enqueue(
                    context,
                    RetrievalPriority::Background,
                    plan_discovery(&request),
                );
            }
            WorkCommand::Query {
                context,
                plan,
                reply,
            } => {
                self.queries.register(context.clone(), reply);
                self.enqueue(context, RetrievalPriority::Enrichment, plan);
            }
        }
    }

    fn apply_control_command(&mut self, command: ControlCommand) {
        match command {
            ControlCommand::SetDataUsage(level) => {
                self.max_concurrent = max_concurrent_requests(level)
            }
            ControlCommand::ResetSession { reply } => self.reset_session(reply),
        }
    }

    fn open_feed(&mut self, context: FeedContext, request: DiscoveryRequest) {
        self.cancel_hunt(&context);
        self.clear_feed_retry(&context);
        self.cancel_context_work(&context);
        self.feeds.close(&context);
        self.feeds.open(context.clone(), request.clone());
        self.queue.focus(context.clone());
        self.enqueue(
            context,
            RetrievalPriority::Interactive,
            plan_discovery(&request),
        );
    }

    fn load_more(&mut self, context: FeedContext, older_than: Option<Timestamp>) {
        if self.feeds.is_continuous(&context) {
            self.cancel_hunt(&context);
        }
        let Some(request) = self.feeds.older_page_request(&context, older_than) else {
            return;
        };
        self.queue.focus(context.clone());
        self.enqueue(
            context,
            RetrievalPriority::Interactive,
            plan_discovery(&request),
        );
    }

    pub(crate) fn enqueue(
        &mut self,
        context: FeedContext,
        priority: RetrievalPriority,
        plan: QueryPlan,
    ) {
        if priority == RetrievalPriority::Background {
            let _ = self.outcomes.send(RetrievalOutcome::Started {
                context: context.clone(),
            });
        }
        self.queue
            .push(RetrievalRequest { context, priority }, plan);
    }
}
