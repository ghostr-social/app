use super::{ChunkAttempt, DownloadWorkers};

impl DownloadWorkers {
    pub(in crate::manager) fn body_renewal_delta(&self, attempt: &ChunkAttempt, through: u64) -> Option<u64> {
        self.active.body_renewal_delta(attempt, through)
    }

    pub(in crate::manager) fn commit_body_renewal(&mut self, attempt: &ChunkAttempt, delta: u64) {
        self.active.commit_body_renewal(attempt, delta);
    }
}
