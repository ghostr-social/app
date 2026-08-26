//! Account-session teardown owned by the discovery scheduler.

use crate::scheduler::SchedulerWorker;
use tokio::sync::oneshot;

impl SchedulerWorker {
    pub(super) fn reset_session(&mut self, reply: oneshot::Sender<()>) {
        for task in core::mem::take(&mut self.tasks).into_values() {
            task.abort.abort();
        }
        for task in core::mem::take(&mut self.hunts).into_values() {
            task.abort();
        }
        self.queue.reset_session();
        self.feeds.reset_session();
        self.deferred_reposts.reset();
        self.queries.reset_session();
        self.retry_attempts.clear();
        self.pending_feed_retries.clear();
        self.pending_feed_hunts.clear();
        let _ = reply.send(());
    }
}

impl Drop for SchedulerWorker {
    fn drop(&mut self) {
        for task in self.tasks.values() {
            task.abort.abort();
        }
        for hunt in self.hunts.values() {
            hunt.abort();
        }
    }
}
