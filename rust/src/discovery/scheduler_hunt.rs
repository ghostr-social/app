//! Rust-owned lifecycle for active search and hashtag feeds.

use crate::discovery::discovery_scheduler::{DiscoveryCommand, SchedulerWorker};
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::discovery::scheduler_feeds::{QueryHuntAction, QUERY_HUNT_BACKOFF};
use crate::discovery::search_queries::plan_discovery;

impl SchedulerWorker {
    pub(crate) fn advance_query(&mut self, context: FeedContext) {
        match self.feeds.hunt_action(&context) {
            Some(QueryHuntAction::OlderNow) => self.prefetch(context),
            Some(QueryHuntAction::HeadLater) => self.schedule_head(context),
            None => {}
        }
    }

    pub(crate) fn continue_query(&mut self, context: FeedContext, head: bool) {
        self.hunts.remove(&context);
        if !self.feeds.is_query(&context) || self.query_busy(&context) {
            return;
        }
        if head {
            self.refresh_head(context);
        } else {
            self.prefetch(context);
        }
    }

    pub(crate) fn close_feed(&mut self, context: FeedContext) {
        self.cancel_hunt(&context);
        self.queue.remove(&context);
        self.feeds.close(&context);
        let tasks = self.context_tasks(&context);
        for task_id in tasks {
            if let Some(task) = self.tasks.remove(&task_id) {
                task.abort.abort();
            }
        }
    }

    pub(crate) fn cancel_hunt(&mut self, context: &FeedContext) {
        if let Some(task) = self.hunts.remove(context) {
            task.abort();
        }
    }

    fn refresh_head(&mut self, context: FeedContext) {
        let Some(request) = self.feeds.head_request(&context) else {
            return;
        };
        self.enqueue(
            context,
            RetrievalPriority::Background,
            plan_discovery(&request),
        );
    }

    fn schedule_head(&mut self, context: FeedContext) {
        self.cancel_hunt(&context);
        let sender = self.command_sender.clone();
        let scheduled = context.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(QUERY_HUNT_BACKOFF).await;
            if let Some(sender) = sender.upgrade() {
                let _ = sender.send(DiscoveryCommand::ContinueQuery {
                    context: scheduled,
                    head: true,
                });
            }
        });
        self.hunts.insert(context, task.abort_handle());
    }

    fn query_busy(&self, context: &FeedContext) -> bool {
        let queued = self.queue.has_pending(context);
        self.feeds.query_state(context, queued).busy
    }

    fn context_tasks(&self, context: &FeedContext) -> Vec<u64> {
        self.tasks
            .iter()
            .filter_map(|(id, task)| (task.context == *context).then_some(*id))
            .collect()
    }
}
