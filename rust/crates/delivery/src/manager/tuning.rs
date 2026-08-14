use crate::manager::retry::RetryPolicy;
use ghostr_partial_store::partial_range_store::capacity::DEFAULT_RECHECK;
use std::time::Duration;

/// Operational knobs outside the engine's tuning table.
#[derive(Clone, Copy, Debug)]
pub struct DeliveryTuning {
    /// Concurrent HEAD probes for unknown-size posts.
    pub probe_concurrency: usize,
    /// Backoff ladder and give-up budgets for failing sources.
    pub retry: RetryPolicy,
    /// Quiet period before persisting the host-stats snapshot.
    pub stats_debounce: Duration,
    /// Delay before one free-space recheck after a store refusal.
    pub store_pressure_pause: Duration,
}

impl Default for DeliveryTuning {
    fn default() -> Self {
        Self {
            probe_concurrency: 2,
            retry: RetryPolicy::default(),
            stats_debounce: Duration::from_secs(2),
            store_pressure_pause: DEFAULT_RECHECK,
        }
    }
}
