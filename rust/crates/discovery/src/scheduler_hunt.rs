//! Rust-owned lifecycle for active search and hashtag feeds.

use crate::discovery_scheduler::{DiscoveryCommand, SchedulerWorker};
use crate::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::scheduler_feeds::{FeedHuntAction, FEED_REFRESH_BACKOFF};
use crate::search_queries::plan_discovery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HuntToken(pub(crate) u64);

impl SchedulerWorker {
    pub(crate) fn advance_feed_hunt(&mut self, context: FeedContext) {
        match self.feeds.hunt_action(&context) {
            Some(FeedHuntAction::OlderNow) => self.prefetch(context),
            Some(FeedHuntAction::HeadLater) => self.schedule_head(context),
            None => {}
        }
    }

    pub(crate) fn continue_feed(&mut self, context: FeedContext, head: bool, token: HuntToken) {
        if self.pending_feed_hunts.get(&context) != Some(&token) {
            return;
        }
        self.pending_feed_hunts.remove(&context);
        self.hunts.remove(&context);
        if !self.feeds.is_continuous(&context) || self.feed_busy(&context) {
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
        self.clear_feed_retry(&context);
        self.cancel_context_work(&context);
        self.feeds.close(&context);
    }

    pub(crate) fn cancel_context_work(&mut self, context: &FeedContext) {
        self.queue.remove(context);
        let tasks = self.context_tasks(context);
        for task_id in tasks {
            if let Some(task) = self.tasks.remove(&task_id) {
                task.abort.abort();
            }
        }
    }

    pub(crate) fn cancel_hunt(&mut self, context: &FeedContext) {
        self.pending_feed_hunts.remove(context);
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
        let token = self.next_hunt_token();
        self.pending_feed_hunts.insert(context.clone(), token);
        let sender = self.command_sender.clone();
        let scheduled = context.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(FEED_REFRESH_BACKOFF).await;
            if let Some(sender) = sender.upgrade() {
                let _ = sender.send(DiscoveryCommand::ContinueFeed {
                    context: scheduled,
                    head: true,
                    token,
                });
            }
        });
        self.hunts.insert(context, task.abort_handle());
    }

    pub(crate) fn feed_busy(&self, context: &FeedContext) -> bool {
        let queued = self.queue.has_pending(context);
        self.feeds.query_state(context, queued).busy
    }

    fn context_tasks(&self, context: &FeedContext) -> Vec<u64> {
        self.tasks
            .iter()
            .filter_map(|(id, task)| (task.context == *context).then_some(*id))
            .collect()
    }

    pub(crate) fn next_hunt_token(&mut self) -> HuntToken {
        self.next_hunt_token = self.next_hunt_token.wrapping_add(1);
        HuntToken(self.next_hunt_token)
    }
}
