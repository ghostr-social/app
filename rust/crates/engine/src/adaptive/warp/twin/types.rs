use core::hash::{Hash, Hasher as _};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TwinConfig {
    pub(crate) particles: u16,
    pub(crate) tail_bps: u16,
}

impl TwinConfig {
    pub(crate) const fn new(particles: u16, tail_bps: u16) -> Self {
        Self {
            particles,
            tail_bps: if tail_bps > 9_999 { 9_999 } else { tail_bps },
        }
    }
}

impl Default for TwinConfig {
    fn default() -> Self {
        Self::new(32, 9_500)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TwinEpochs {
    pub evidence: u64,
    pub model: u64,
    pub budget: u64,
}

impl TwinEpochs {
    pub const fn new(evidence: u64, model: u64, budget: u64) -> Self {
        Self {
            evidence,
            model,
            budget,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TwinState {
    pub(super) current_buffer_ms: u64,
    pub(super) ready_coverage_ms: u64,
    pub(super) cache_bytes: u64,
    pub(super) throughput_bps: u64,
    pub(super) rtt_ms: u64,
    pub(super) request_slots: u16,
    pub(super) forward_swipes_per_minute: u16,
}

impl TwinState {
    pub(crate) const fn new(
        current_buffer_ms: u64,
        throughput_bps: u64,
        rtt_ms: u64,
        request_slots: u16,
    ) -> Self {
        Self {
            current_buffer_ms,
            ready_coverage_ms: 0,
            cache_bytes: 0,
            throughput_bps,
            rtt_ms,
            request_slots,
            forward_swipes_per_minute: 0,
        }
    }

    pub(crate) const fn with_ready_coverage(mut self, milliseconds: u64) -> Self {
        self.ready_coverage_ms = milliseconds;
        self
    }

    pub(crate) const fn with_cache_bytes(mut self, bytes: u64) -> Self {
        self.cache_bytes = bytes;
        self
    }

    pub(crate) const fn with_swipe_rate(mut self, swipes_per_minute: u16) -> Self {
        self.forward_swipes_per_minute = swipes_per_minute;
        self
    }

    pub(super) fn signature(self) -> TwinStateSignature {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        TwinStateSignature(hasher.finish())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TwinStateSignature(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TwinEvaluation {
    pub(crate) expected_score_micros: i64,
    pub(crate) expected_visible_delay_ms: u64,
    pub(crate) p95_visible_delay_ms: u64,
    pub(crate) p99_visible_delay_ms: u64,
    pub(crate) cvar_visible_delay_ms: u64,
    pub(crate) on_time_probability_bps: u16,
    pub(crate) expected_ready_coverage_ms: u64,
    pub(crate) expected_cache_bytes: u64,
    pub(crate) common_random_seed: u64,
}
