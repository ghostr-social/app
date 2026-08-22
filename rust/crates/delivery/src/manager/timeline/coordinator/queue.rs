use super::TimelineCoordinator;
use crate::manager::timeline::attempts::TimelineAttempt;
use crate::manager::timeline::job;
use ghostr_engine::PostId;

impl TimelineCoordinator {
    pub(crate) fn dispatch(&mut self, priority: &[PostId]) {
        self.priority = priority.to_vec();
        self.reorder();
        self.launch_pending();
    }

    pub(super) fn can_queue(&self, post: &PostId) -> bool {
        self.running < self.maximum
            || self.pending.contains_key(post)
            || self.pending.len() < super::TIMELINE_PENDING
    }

    pub(super) fn queue(&mut self, attempt: TimelineAttempt) {
        let post = attempt.post().clone();
        if self.pending.insert(post.clone(), attempt).is_none() {
            self.order.push_back(post);
        }
    }

    pub(super) fn launch_pending(&mut self) {
        while self.running < self.maximum {
            let Some(post) = self.order.pop_front() else {
                return;
            };
            let Some(attempt) = self.pending.remove(&post) else {
                continue;
            };
            self.launch(attempt);
        }
    }

    pub(super) fn reorder(&mut self) {
        let mut queued: Vec<_> = self.order.drain(..).collect();
        for post in &self.priority {
            if let Some(index) = queued.iter().position(|queued| queued == post) {
                self.order.push_back(queued.remove(index));
            }
        }
        self.order.extend(queued);
    }

    fn launch(&mut self, attempt: TimelineAttempt) {
        self.running += 1;
        let store = self.store.clone();
        let parser = self.parser.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let result = job::run(attempt, store, parser).await;
            let _ = sender.send(result).await;
        });
    }

    pub(super) fn job_finished(&mut self) {
        self.running = self.running.saturating_sub(1);
    }
}
