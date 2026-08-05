//! Account-session teardown owned by the discovery scheduler.

use crate::discovery::discovery_scheduler::SchedulerWorker;
use tokio::sync::oneshot;

impl SchedulerWorker {
    pub(crate) fn reset_session(&mut self, reply: oneshot::Sender<()>) {
        for (_, task) in self.tasks.drain() {
            task.abort.abort();
        }
        for (_, task) in self.hunts.drain() {
            task.abort();
        }
        self.queue.reset_session();
        self.feeds.reset_session();
        self.queries.reset_session();
        self.retry_attempts.clear();
        self.pending_feed_retries.clear();
        self.pending_query_hunts.clear();
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
