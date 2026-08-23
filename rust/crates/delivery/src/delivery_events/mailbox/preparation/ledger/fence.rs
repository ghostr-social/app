use crate::delivery_events::{PlayerPreparationFollowup, PlayerPreparationReport};
use ghostr_engine::PostId;

#[derive(Clone, Debug)]
pub(super) struct AttemptFence {
    post: PostId,
    client_epoch: u64,
    max_attempt: u64,
}

impl AttemptFence {
    pub(super) fn capture(report: &PlayerPreparationReport) -> Self {
        Self {
            post: report.post().clone(),
            client_epoch: report.client_epoch(),
            max_attempt: report.attempt_generation(),
        }
    }

    pub(super) fn matches(&self, report: &PlayerPreparationReport) -> bool {
        self.post == *report.post() && self.client_epoch == report.client_epoch()
    }

    pub(super) fn fences(&self, report: &PlayerPreparationFollowup) -> bool {
        self.post == *report.post()
            && self.client_epoch == report.client_epoch()
            && self.max_attempt >= report.attempt_generation()
    }

    pub(super) const fn max_attempt(&self) -> u64 {
        self.max_attempt
    }
}
