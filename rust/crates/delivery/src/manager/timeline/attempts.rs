use super::TimelineEvidence;
use core::sync::atomic::{AtomicBool, Ordering};
use ghostr_engine::PostId;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineAttemptDisposition {
    Retryable,
    Terminal,
}

#[derive(Clone, Debug)]
pub(crate) struct TimelineAttempt {
    post: PostId,
    id: u64,
    evidence: TimelineEvidence,
    cancelled: Arc<AtomicBool>,
}

#[derive(Default)]
pub(crate) struct TimelineAttempts {
    active: BTreeMap<PostId, TimelineAttempt>,
    terminal: BTreeMap<PostId, TimelineEvidence>,
    next_id: u64,
}

impl TimelineAttempts {
    pub(super) fn matches(&self, post: &PostId, evidence: &TimelineEvidence) -> bool {
        self.terminal
            .get(post)
            .is_some_and(|known| known.same_parse(evidence))
            || self
                .active
                .get(post)
                .is_some_and(|attempt| attempt.evidence.same_parse(evidence))
    }

    pub(crate) fn start(
        &mut self,
        post: PostId,
        evidence: TimelineEvidence,
    ) -> Option<TimelineAttempt> {
        if self.matches(&post, &evidence) {
            return None;
        }
        self.cancel_active(&post);
        self.terminal.remove(&post);
        let attempt = self.next(post.clone(), evidence);
        self.active.insert(post, attempt.clone());
        Some(attempt)
    }

    pub(crate) fn finish(
        &mut self,
        attempt: &TimelineAttempt,
        disposition: TimelineAttemptDisposition,
    ) -> bool {
        if self.active.get(&attempt.post).map(|active| active.id) != Some(attempt.id) {
            return false;
        }
        self.active.remove(&attempt.post);
        if disposition == TimelineAttemptDisposition::Terminal {
            self.terminal
                .insert(attempt.post.clone(), attempt.evidence.clone());
        }
        true
    }

    pub(super) fn retain_active(&mut self, posts: &HashSet<PostId>) {
        let stale: Vec<_> = self
            .active
            .keys()
            .filter(|post| !posts.contains(*post))
            .cloned()
            .collect();
        for post in stale {
            self.cancel_active(&post);
        }
    }

    pub(super) fn retain_history(&mut self, posts: &HashSet<PostId>) {
        self.terminal.retain(|post, _| posts.contains(post));
    }

    pub(super) fn clear(&mut self) {
        for attempt in self.active.values() {
            attempt.cancel();
        }
        self.active.clear();
        self.terminal.clear();
    }

    pub(super) fn invalidate(&mut self, post: &PostId) {
        self.cancel_active(post);
        self.terminal.remove(post);
    }

    pub(super) fn is_current(&self, attempt: &TimelineAttempt) -> bool {
        self.active.get(&attempt.post).map(|active| active.id) == Some(attempt.id)
    }

    fn cancel_active(&mut self, post: &PostId) {
        if let Some(attempt) = self.active.remove(post) {
            attempt.cancel();
        }
    }

    fn next(&mut self, post: PostId, evidence: TimelineEvidence) -> TimelineAttempt {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("timeline attempt identity exhausted");
        TimelineAttempt {
            post,
            id: self.next_id,
            evidence,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl TimelineAttempt {
    pub(super) fn post(&self) -> &PostId {
        &self.post
    }

    pub(super) fn evidence(&self) -> &TimelineEvidence {
        &self.evidence
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(super) fn control(&self) -> Arc<AtomicBool> {
        std::sync::Arc::clone(&self.cancelled)
    }

    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
