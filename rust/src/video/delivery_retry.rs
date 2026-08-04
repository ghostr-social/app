//! Per-source retry policy: exponential backoff with jitter, an
//! attempt budget that runs out far sooner for permanent-class
//! failures, and a long retirement cooldown for sources that spend it.
//!
//! Without this the manager re-dialled a failing source on every event
//! after a flat three-second pause, forever — a device pass recorded
//! 174 identical DNS failures against one host in ten minutes. A
//! retired source is dropped from its post's candidate list, so the
//! post falls back to another mirror or becomes terminal instead of
//! being rescheduled by every reconcile pass.

use crate::engine::PostId;
use crate::video::delivery_failure::FailureClass;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time::Instant;

/// One post's use of one source URL: the unit attempts are counted on.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Source {
    pub post: PostId,
    pub url: String,
}

impl Source {
    pub fn new(post: PostId, url: String) -> Self {
        Self { post, url }
    }
}

/// The backoff ladder and the attempt budgets it is spent from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RetryPolicy {
    /// Pause after the first failed attempt; doubles from there.
    pub base: Duration,
    /// Ceiling for the doubling, before jitter.
    pub max: Duration,
    /// Fraction of the pause spread randomly around it, in `[0, 1]`.
    pub jitter: f64,
    /// Attempts a source gets against transient failures.
    pub transient_attempts: u32,
    /// Attempts a source gets against permanent-class failures.
    pub permanent_attempts: u32,
    /// How long a source stays retired once its budget ran out. Long
    /// enough to stop a storm, short enough that a passing outage does
    /// not silence a host for the rest of the session.
    pub revive_after: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            base: Duration::from_secs(3),
            max: Duration::from_secs(300),
            jitter: 0.25,
            transient_attempts: 5,
            permanent_attempts: 2,
            revive_after: Duration::from_secs(600),
        }
    }
}

/// What the policy grants after one failed attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Retry {
    /// Try this source again, but not before the given pause.
    After(Duration),
    /// The source spent its budget; stop dialling it.
    GiveUp,
}

/// The attempt ledger for every (post, source) pair that has failed.
pub struct RetryBook {
    policy: RetryPolicy,
    attempts: HashMap<Source, u32>,
    retired: HashMap<Source, Instant>,
    cooling: HashSet<PostId>,
}

impl RetryBook {
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            attempts: HashMap::new(),
            retired: HashMap::new(),
            cooling: HashSet::new(),
        }
    }

    /// Charges one failed attempt to `source` and answers with the
    /// pause before the next one, or with giving up.
    pub fn note_failure(&mut self, source: Source, class: FailureClass) -> Retry {
        if self.is_retired(&source) {
            return Retry::GiveUp;
        }
        let attempts = self.charge(&source);
        match attempts >= self.budget(class) {
            true => self.retire(source),
            false => Retry::After(self.backoff(attempts)),
        }
    }

    /// A source that answered again starts from a clean slate.
    pub fn note_success(&mut self, source: &Source) {
        self.attempts.remove(source);
        self.retired.remove(source);
    }

    /// Whether `source` is still inside its retirement cooldown.
    pub fn is_retired(&self, source: &Source) -> bool {
        self.retired
            .get(source)
            .is_some_and(|until| *until > Instant::now())
    }

    /// The post's candidate URLs that are still worth dialling, in the
    /// order they were advertised.
    pub fn live_urls(&self, post: &PostId, urls: &[String]) -> Vec<String> {
        urls.iter()
            .filter(|url| !self.is_retired(&Source::new(post.clone(), (*url).clone())))
            .cloned()
            .collect()
    }

    /// Whether every source of the post is retired: nothing can be
    /// fetched for it, so it is terminal until one of them revives.
    pub fn all_retired(&self, post: &PostId, urls: &[String]) -> bool {
        !urls.is_empty() && self.live_urls(post, urls).is_empty()
    }

    /// Marks the post as pausing between attempts. `false` when a
    /// pause was already running, so timers are not stacked.
    pub fn cool_down(&mut self, post: PostId) -> bool {
        self.cooling.insert(post)
    }

    pub fn warm_up(&mut self, post: &PostId) {
        self.cooling.remove(post);
    }

    pub fn is_cooling(&self, post: &PostId) -> bool {
        self.cooling.contains(post)
    }

    fn charge(&mut self, source: &Source) -> u32 {
        let attempts = self.attempts.entry(source.clone()).or_insert(0);
        *attempts += 1;
        *attempts
    }

    fn budget(&self, class: FailureClass) -> u32 {
        let attempts = match class {
            FailureClass::Permanent => self.policy.permanent_attempts,
            FailureClass::Transient => self.policy.transient_attempts,
        };
        attempts.max(1)
    }

    /// Retiring also sweeps every revived entry, keeping the ledger
    /// bounded by the failures of the last revival window.
    fn retire(&mut self, source: Source) -> Retry {
        let now = Instant::now();
        self.retired.retain(|_, until| *until > now);
        self.attempts.remove(&source);
        self.retired.insert(source, now + self.policy.revive_after);
        Retry::GiveUp
    }

    /// Doubling ladder capped at `max`, then spread by jitter so a feed
    /// full of posts on one broken host does not retry in lockstep.
    fn backoff(&self, attempts: u32) -> Duration {
        let steps = attempts.saturating_sub(1).min(16);
        let grown = self.policy.base.saturating_mul(1u32 << steps);
        jittered(grown.min(self.policy.max), self.policy.jitter)
    }
}

fn jittered(wait: Duration, jitter: f64) -> Duration {
    let spread = jitter.clamp(0.0, 1.0);
    if spread == 0.0 {
        return wait;
    }
    wait.mul_f64(1.0 + spread * (rand::random::<f64>() * 2.0 - 1.0))
}
