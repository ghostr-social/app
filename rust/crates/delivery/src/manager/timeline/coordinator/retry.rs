use crate::manager::timeline::attempts::TimelineAttempt;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time::Instant;

const RETRY_BASE: Duration = Duration::from_millis(100);
const MAXIMUM_BACKOFF_SHIFT: u32 = 4;

pub(super) struct TimelineRetries {
    waiting: HashMap<PostId, WaitingRetry>,
    failures: HashMap<PostId, u32>,
}

struct WaitingRetry {
    attempt: TimelineAttempt,
    due: Instant,
}

impl TimelineRetries {
    pub(super) fn new() -> Self {
        Self {
            waiting: HashMap::new(),
            failures: HashMap::new(),
        }
    }

    pub(super) fn defer(&mut self, attempt: TimelineAttempt) {
        let post = attempt.post().clone();
        let failures = self.failures.entry(post.clone()).or_insert(0);
        *failures = failures.saturating_add(1);
        let shift = failures.saturating_sub(1).min(MAXIMUM_BACKOFF_SHIFT);
        let wait = RETRY_BASE.saturating_mul(1_u32 << shift);
        self.waiting.insert(
            post,
            WaitingRetry {
                attempt,
                due: Instant::now() + wait,
            },
        );
    }

    pub(super) fn next_deadline(&self) -> Option<Instant> {
        self.waiting.values().map(|retry| retry.due).min()
    }

    pub(super) fn take_due(&mut self) -> Vec<TimelineAttempt> {
        let now = Instant::now();
        let posts: Vec<_> = self
            .waiting
            .iter()
            .filter(|(_, retry)| retry.due <= now)
            .map(|(post, _)| post.clone())
            .collect();
        posts
            .into_iter()
            .filter_map(|post| self.waiting.remove(&post).map(|retry| retry.attempt))
            .collect()
    }

    pub(super) fn reset(&mut self, post: &PostId) {
        self.waiting.remove(post);
        self.failures.remove(post);
    }

    pub(super) fn retain(&mut self, posts: &HashSet<PostId>) {
        self.waiting.retain(|post, _| posts.contains(post));
        self.failures.retain(|post, _| posts.contains(post));
    }

    pub(super) fn clear(&mut self) {
        self.waiting.clear();
        self.failures.clear();
    }
}
