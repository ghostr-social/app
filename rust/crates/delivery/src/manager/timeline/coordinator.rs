use super::attempts::{TimelineAttempt, TimelineAttemptDisposition, TimelineAttempts};
use super::outcome::{TimelineJobOutcome, TimelineResult};
use super::parser::{ProductionTimelineParser, TimelineParser};
use super::TimelineEvidence;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc;

mod publication;
mod queue;
mod receive;
mod retry;

use publication::TimelinePublications;
use retry::TimelineRetries;

const TIMELINE_WORKERS: usize = 2;
const TIMELINE_PENDING: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineSchedule {
    Current,
    Deferred,
    Started,
}

pub(crate) struct TimelineCoordinator {
    attempts: TimelineAttempts,
    pending: HashMap<PostId, TimelineAttempt>,
    order: VecDeque<PostId>,
    priority: Vec<PostId>,
    running: usize,
    maximum: usize,
    store: Arc<PartialRangeStore>,
    parser: Arc<dyn TimelineParser>,
    sender: mpsc::Sender<TimelineResult>,
    receiver: mpsc::Receiver<TimelineResult>,
    completed: VecDeque<TimelineResult>,
    wake_ready: Option<TimelineResult>,
    retries: TimelineRetries,
    publications: TimelinePublications,
}

impl TimelineCoordinator {
    pub(crate) fn new(store: Arc<PartialRangeStore>) -> Self {
        Self::with_parser(store, Arc::new(ProductionTimelineParser), TIMELINE_WORKERS)
    }

    pub(crate) fn with_parser(
        store: Arc<PartialRangeStore>,
        parser: Arc<dyn TimelineParser>,
        maximum: usize,
    ) -> Self {
        let maximum = maximum.max(1);
        let (sender, receiver) = mpsc::channel(maximum);
        Self {
            attempts: TimelineAttempts::default(),
            pending: HashMap::new(),
            order: VecDeque::new(),
            priority: Vec::new(),
            running: 0,
            maximum,
            store,
            parser,
            sender,
            receiver,
            completed: VecDeque::new(),
            wake_ready: None,
            retries: TimelineRetries::new(),
            publications: TimelinePublications::default(),
        }
    }

    pub(crate) fn schedule(
        &mut self,
        post: PostId,
        evidence: TimelineEvidence,
    ) -> TimelineSchedule {
        if self.attempts.matches(&post, &evidence) {
            return TimelineSchedule::Current;
        }
        self.cancel_work(&post);
        if !self.can_queue(&post) {
            return TimelineSchedule::Deferred;
        }
        let Some(attempt) = self.attempts.start(post.clone(), evidence) else {
            return TimelineSchedule::Current;
        };
        if self.pending.insert(post.clone(), attempt).is_none() {
            self.order.push_back(post);
        }
        TimelineSchedule::Started
    }

    pub(crate) fn stage(&mut self, result: TimelineResult) {
        self.completed.push_back(result);
    }

    pub(super) fn take_completed(&mut self) -> Vec<TimelineResult> {
        self.completed.drain(..).collect()
    }

    pub(crate) fn validate(
        &mut self,
        result: TimelineResult,
        current: Option<&TimelineEvidence>,
    ) -> Option<TimelineJobOutcome> {
        let matches =
            current.is_some_and(|evidence| evidence.same_parse(result.attempt.evidence()));
        if !matches || !self.attempts.is_current(&result.attempt) {
            let _ = self
                .attempts
                .finish(&result.attempt, TimelineAttemptDisposition::Retryable);
            return None;
        }
        let disposition = match result.outcome {
            TimelineJobOutcome::Terminal(_) => TimelineAttemptDisposition::Terminal,
            _ => TimelineAttemptDisposition::Retryable,
        };
        if !self.attempts.finish(&result.attempt, disposition) {
            return None;
        }
        self.retries.reset(result.attempt.post());
        Some(result.outcome)
    }

    pub(super) fn retain_active(&mut self, posts: &HashSet<PostId>) {
        self.attempts.retain_active(posts);
        self.pending.retain(|post, _| posts.contains(post));
        self.order.retain(|post| posts.contains(post));
        self.completed
            .retain(|result| posts.contains(result.post()));
        if self
            .wake_ready
            .as_ref()
            .is_some_and(|result| !posts.contains(result.post()))
        {
            self.wake_ready = None;
        }
        self.retries.retain(posts);
    }

    pub(crate) fn retain_history(&mut self, posts: &HashSet<PostId>) {
        self.attempts.retain_history(posts);
        self.publications.retain(posts);
    }

    pub(crate) fn clear(&mut self) {
        self.attempts.clear();
        self.pending.clear();
        self.order.clear();
        self.priority.clear();
        self.completed.clear();
        self.wake_ready = None;
        self.retries.clear();
        self.publications.clear();
    }

    pub(crate) fn invalidate(&mut self, post: &PostId) {
        self.cancel_work(post);
        self.publications.remove(post);
    }

    fn cancel_work(&mut self, post: &PostId) {
        self.attempts.invalidate(post);
        self.pending.remove(post);
        self.order.retain(|queued| queued != post);
        self.completed.retain(|result| result.post() != post);
        if self.wake_ready.as_ref().map(TimelineResult::post) == Some(post) {
            self.wake_ready = None;
        }
        self.retries.reset(post);
    }
}
