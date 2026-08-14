//! Retry budgets and the exponential backoff policy they grant.

use std::time::Duration;

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
    /// How long a source stays retired once its budget ran out.
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

impl RetryPolicy {
    /// Answers the capped doubling ladder with bounded jitter.
    pub(super) fn backoff(&self, attempts: u32) -> Duration {
        let steps = attempts.saturating_sub(1).min(16);
        let grown = self.base.saturating_mul(1u32 << steps);
        jittered(grown.min(self.max), self.jitter)
    }
}

fn jittered(wait: Duration, jitter: f64) -> Duration {
    let spread = jitter.clamp(0.0, 1.0);
    if spread == 0.0 {
        return wait;
    }
    wait.mul_f64(1.0 + spread * (rand::random::<f64>() * 2.0 - 1.0))
}

/// What the policy grants after one failed attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Retry {
    /// Try this source again, but not before the given pause.
    After(Duration),
    /// The source spent its budget; stop dialling it.
    GiveUp,
}
