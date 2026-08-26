use super::TimelineCoordinator;
use crate::manager::timeline::outcome::{TimelineJobOutcome, TimelineResult, TimelineRetry};
use ghostr_engine::PostId;

impl TimelineCoordinator {
    pub(crate) async fn recv(&mut self) -> Option<TimelineResult> {
        loop {
            let result = self.next_result().await?;
            self.job_finished();
            if !self.defer_retry(&result) {
                return Some(result);
            }
        }
    }

    fn try_recv(&mut self) -> Option<TimelineResult> {
        self.launch_due();
        loop {
            let result = self.receiver.try_recv().ok()?;
            self.job_finished();
            if !self.defer_retry(&result) {
                return Some(result);
            }
        }
    }

    pub(crate) fn prepare_wake(&mut self) -> bool {
        if self.wake_ready.is_none() {
            self.wake_ready = self.try_recv();
        }
        self.wake_ready.is_some()
    }

    pub(crate) fn take_wake(&mut self) -> Option<TimelineResult> {
        self.wake_ready.take()
    }

    async fn next_result(&mut self) -> Option<TimelineResult> {
        loop {
            self.launch_due();
            let Some(deadline) = self.retries.next_deadline() else {
                return self.receiver.recv().await;
            };
            tokio::select! {
                result = self.receiver.recv() => return result,
                () = tokio::time::sleep_until(deadline) => {}
            }
        }
    }

    fn defer_retry(&mut self, result: &TimelineResult) -> bool {
        let TimelineJobOutcome::Retryable(retry) = &result.outcome else {
            return false;
        };
        note_retry(result.attempt.post(), retry);
        if self.attempts.is_current(&result.attempt) && !result.attempt.is_cancelled() {
            self.retries.defer(result.attempt.clone());
        }
        true
    }

    fn launch_due(&mut self) {
        for attempt in self.retries.take_due() {
            if self.attempts.is_current(&attempt) && !attempt.is_cancelled() {
                self.queue(attempt);
            }
        }
        self.reorder();
        self.launch_pending();
    }
}

fn note_retry(post: &PostId, retry: &TimelineRetry) {
    match retry {
        TimelineRetry::Missing => log::debug!("Timeline bytes unavailable for {}", post.as_str()),
        TimelineRetry::Read(error) => {
            log::debug!("Timeline read failed for {}: {error}", post.as_str());
        }
        TimelineRetry::Worker(error) => {
            log::warn!("Timeline worker failed for {}: {error}", post.as_str());
        }
    }
}
