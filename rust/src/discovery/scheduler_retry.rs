//! Bounded retry ladder for canonical feeds whose relay page failed.

use crate::discovery::discovery_scheduler::RetrievalPurpose;
use crate::discovery::discovery_scheduler::{DiscoveryCommand, SchedulerWorker};
use crate::discovery::feed_cursor::playable_cursor;
use crate::discovery::plan_executor::PlanFailure;
use crate::discovery::retrieval_queue::{FeedContext, RetrievalPriority};
use crate::discovery::scheduler_hunt::HuntToken;
use crate::discovery::search_queries::plan_discovery;
use nostr_sdk::Event;
use std::time::Duration;

const RETRY_DELAYS: [Duration; 5] = [
    Duration::from_millis(500),
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(4),
    Duration::from_secs(8),
];
const RETRY_STAGGER_STEP: Duration = Duration::from_millis(25);
const RETRY_STAGGER_SLOTS: u32 = 8;

pub(crate) fn retry_delay(context: &FeedContext, attempt: usize) -> Duration {
    let base = RETRY_DELAYS[attempt.min(RETRY_DELAYS.len() - 1)];
    let checksum = context
        .as_str()
        .bytes()
        .fold(0_u32, |total, byte| total.wrapping_add(u32::from(byte)));
    let slot = (checksum % RETRY_STAGGER_SLOTS) + 1;
    base + RETRY_STAGGER_STEP * slot
}

pub(crate) fn should_retry_feed(
    result: &Result<Vec<Event>, PlanFailure>,
    purpose: RetrievalPurpose,
    has_playable: bool,
) -> bool {
    if purpose != RetrievalPurpose::Head || has_playable {
        return false;
    }
    match result {
        Err(_) => true,
        Ok(events) => playable_cursor(events).is_none(),
    }
}

impl SchedulerWorker {
    pub(crate) fn advance_feed(
        &mut self,
        context: FeedContext,
        retry: bool,
        purpose: RetrievalPurpose,
    ) {
        if self.feeds.is_query(&context) {
            return self.advance_query(context);
        }
        if purpose != RetrievalPurpose::Head {
            return;
        }
        if retry {
            self.schedule_feed_retry(context);
        } else {
            self.clear_feed_retry(&context);
        }
    }

    pub(crate) fn continue_feed_retry(&mut self, context: FeedContext, token: HuntToken) {
        if self.pending_feed_retries.get(&context) != Some(&token) {
            return;
        }
        self.pending_feed_retries.remove(&context);
        self.hunts.remove(&context);
        if self.feeds.is_query(&context) {
            return;
        }
        if self.query_busy(&context) {
            return self.defer_feed_retry(context);
        }
        if let Some(request) = self.feeds.head_request(&context) {
            self.enqueue(
                context,
                RetrievalPriority::Background,
                plan_discovery(&request),
            );
        }
    }

    pub(crate) fn clear_feed_retry(&mut self, context: &FeedContext) {
        self.cancel_hunt(context);
        self.retry_attempts.remove(context);
        self.pending_feed_retries.remove(context);
    }

    fn schedule_feed_retry(&mut self, context: FeedContext) {
        self.cancel_hunt(&context);
        let delay = self.next_retry_delay(&context);
        self.start_feed_retry_timer(context, delay);
    }

    fn defer_feed_retry(&mut self, context: FeedContext) {
        self.cancel_hunt(&context);
        let delay = retry_delay(&context, 0);
        self.start_feed_retry_timer(context, delay);
    }

    fn start_feed_retry_timer(&mut self, context: FeedContext, delay: Duration) {
        let token = self.next_hunt_token();
        self.pending_feed_retries.insert(context.clone(), token);
        let sender = self.command_sender.clone();
        let scheduled = context.clone();
        let task = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            if let Some(sender) = sender.upgrade() {
                let _ = sender.send(DiscoveryCommand::RetryFeed {
                    context: scheduled,
                    token,
                });
            }
        });
        self.hunts.insert(context, task.abort_handle());
    }

    fn next_retry_delay(&mut self, context: &FeedContext) -> Duration {
        let attempt = self.retry_attempts.entry(context.clone()).or_default();
        let delay = retry_delay(context, *attempt);
        *attempt = attempt.saturating_add(1);
        delay
    }
}
