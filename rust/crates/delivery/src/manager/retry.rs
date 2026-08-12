//! Per-source retry policy: exponential backoff with jitter, an
//! attempt budget that runs out far sooner for permanent-class
//! failures, and a long retirement cooldown for sources that spend it.
//!
//! Retired sources leave the post's candidate list, allowing mirror
//! fallback or a terminal result instead of endless rescheduling.

use crate::manager::failure::FailureClass;
use ghostr_engine::PostId;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time::Instant;

mod cooldowns;
mod policy;

pub(crate) use cooldowns::CooldownId;
use cooldowns::Cooldowns;
pub use policy::{Retry, RetryPolicy};

/// One post's use of one source URL: the unit attempts are counted on.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Source {
    post: PostId,
    url: String,
}

impl Source {
    pub fn new(post: PostId, url: String) -> Self {
        Self { post, url }
    }
}

/// The attempt ledger for every (post, source) pair that has failed.
pub struct RetryBook {
    policy: RetryPolicy,
    attempts: HashMap<Source, u32>,
    retired: HashMap<Source, Instant>,
    cooldowns: Cooldowns,
}

impl RetryBook {
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            attempts: HashMap::new(),
            retired: HashMap::new(),
            cooldowns: Cooldowns::default(),
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
        self.cooldowns.note_success(&source.post);
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
        let mut live: Vec<_> = urls
            .iter()
            .filter(|url| !self.is_retired(&Source::new(post.clone(), (*url).clone())))
            .cloned()
            .collect();
        live.sort_by_key(|url| self.failure_count(post, url));
        live
    }

    pub fn has_ready_alternative(&self, post: &PostId, failed: &str, urls: &[String]) -> bool {
        self.live_urls(post, urls).iter().any(|url| url != failed)
    }

    /// Whether every source of the post is retired: nothing can be
    /// fetched for it, so it is terminal until one of them revives.
    pub fn all_retired(&self, post: &PostId, urls: &[String]) -> bool {
        !urls.is_empty() && self.live_urls(post, urls).is_empty()
    }

    /// Marks the post as pausing between attempts. `None` when a
    /// pause was already running, so timers are not stacked.
    pub(crate) fn cool_down(&mut self, post: PostId) -> Option<CooldownId> {
        self.cooldowns.begin(post)
    }

    pub(crate) fn representation_changed(&mut self, post: &PostId) {
        self.cooldowns.representation_changed(post);
    }

    pub(crate) fn focus_changed(&mut self, previous: Option<&PostId>, current: Option<&PostId>) {
        self.cooldowns.focus_changed(previous, current);
    }

    pub(crate) fn warm_up(&mut self, post: &PostId, cooldown: CooldownId) -> bool {
        self.cooldowns.finish(post, cooldown)
    }

    pub(crate) fn expedite_demand(&mut self, post: &PostId, offset: u64) -> bool {
        self.cooldowns.expedite_demand(post, offset)
    }

    pub(crate) fn is_cooling(&self, post: &PostId) -> bool {
        self.cooldowns.is_active(post)
    }

    #[cfg(test)]
    pub(crate) fn demand_tracking_units(&self) -> usize {
        self.cooldowns.demand_tracking_units()
    }

    pub(crate) fn clear(&mut self) {
        self.attempts.clear();
        self.retired.clear();
        self.cooldowns.clear();
    }

    /// Completed attempt/retirement history follows hot scheduling
    /// retention. Live cooldowns keep their timer ownership until expiry.
    pub(crate) fn retain_history(&mut self, retained: &HashSet<PostId>) {
        self.attempts
            .retain(|source, _| retained.contains(&source.post));
        self.retired
            .retain(|source, _| retained.contains(&source.post));
        self.cooldowns.retain_demand(retained);
    }

    fn charge(&mut self, source: &Source) -> u32 {
        let attempts = self.attempts.entry(source.clone()).or_insert(0);
        *attempts += 1;
        *attempts
    }

    fn failure_count(&self, post: &PostId, url: &str) -> u32 {
        self.attempts
            .get(&Source::new(post.clone(), url.to_owned()))
            .copied()
            .unwrap_or(0)
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
        self.cooldowns.clear_credit(&source.post);
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
