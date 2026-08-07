//! Discovery command dispatch, kept separate from the scheduler wake loop.

use crate::discovery_scheduler::{
    max_concurrent_requests, DiscoveryCommand, RetrievalOutcome, SchedulerWorker,
};
use crate::retrieval_queue::{FeedContext, RetrievalPriority, RetrievalRequest};
use crate::search_queries::{plan_discovery, QueryPlan};
use crate::video_filters::DiscoveryRequest;
use nostr_sdk::Timestamp;

impl SchedulerWorker {
    pub(crate) fn apply_command(&mut self, command: DiscoveryCommand) {
        match command {
            DiscoveryCommand::Focus(context) => self.queue.focus(context),
            DiscoveryCommand::SetDataUsage(level) => {
                self.max_concurrent = max_concurrent_requests(level)
            }
            DiscoveryCommand::ResetSession { reply } => self.reset_session(reply),
            DiscoveryCommand::OpenFeed { context, request } => self.open_feed(context, request),
            DiscoveryCommand::CloseFeed(context) => self.close_feed(context),
            DiscoveryCommand::ContinueFeed {
                context,
                head,
                token,
            } => self.continue_feed(context, head, token),
            DiscoveryCommand::RetryFeed { context, token } => {
                self.continue_feed_retry(context, token)
            }
            DiscoveryCommand::LoadMore {
                context,
                older_than,
            } => self.load_more(context, older_than),
            DiscoveryCommand::Background { context, request } => {
                self.enqueue(
                    context,
                    RetrievalPriority::Background,
                    plan_discovery(&request),
                );
            }
            DiscoveryCommand::Query {
                context,
                plan,
                reply,
            } => {
                self.queries.register(context.clone(), reply);
                self.enqueue(context, RetrievalPriority::Enrichment, plan);
            }
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
