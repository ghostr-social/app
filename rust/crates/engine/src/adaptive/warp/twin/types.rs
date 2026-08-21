use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TwinConfig {
    pub particles: u16,
    pub tail_bps: u16,
}

impl TwinConfig {
    pub const fn new(particles: u16, tail_bps: u16) -> Self {
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
    pub current_buffer_ms: u64,
    pub ready_coverage_ms: u64,
    pub cache_bytes: u64,
    pub throughput_bps: u64,
    pub rtt_ms: u64,
    pub request_slots: u16,
    pub forward_swipes_per_minute: u16,
}

impl TwinState {
    pub const fn new(
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

    pub const fn with_ready_coverage(mut self, milliseconds: u64) -> Self {
        self.ready_coverage_ms = milliseconds;
        self
    }

    pub const fn with_cache_bytes(mut self, bytes: u64) -> Self {
        self.cache_bytes = bytes;
        self
    }

    pub const fn with_swipe_rate(mut self, swipes_per_minute: u16) -> Self {
        self.forward_swipes_per_minute = swipes_per_minute;
        self
    }

    pub fn signature(self) -> TwinStateSignature {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        TwinStateSignature(hasher.finish())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TwinStateSignature(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TwinEvaluation {
    pub expected_score_micros: i64,
    pub expected_visible_delay_ms: u64,
    pub p95_visible_delay_ms: u64,
    pub p99_visible_delay_ms: u64,
    pub cvar_visible_delay_ms: u64,
    pub on_time_probability_bps: u16,
    pub expected_ready_coverage_ms: u64,
    pub expected_cache_bytes: u64,
    pub common_random_seed: u64,
}
