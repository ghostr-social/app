use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlannerWatchEvidence {
    Unavailable,
    Learned {
        reach_probability_bps: u16,
        play_start_p50_ms: u64,
        play_start_p95_ms: u64,
        play_start_p99_ms: u64,
        probability_by_commitment_bps: u16,
        current_safety_deadline_ms: Option<u64>,
    },
}

impl PlannerWatchEvidence {
    pub const fn learned(
        reach_probability_bps: u16,
        play_start_p50_ms: u64,
        play_start_p95_ms: u64,
        play_start_p99_ms: u64,
        probability_by_commitment_bps: u16,
        current_safety_deadline_ms: Option<u64>,
    ) -> Self {
        let p95 = maximum(play_start_p50_ms, play_start_p95_ms);
        Self::Learned {
            reach_probability_bps: clamp_bps(reach_probability_bps),
            play_start_p50_ms,
            play_start_p95_ms: p95,
            play_start_p99_ms: maximum(p95, play_start_p99_ms),
            probability_by_commitment_bps: clamp_bps(probability_by_commitment_bps),
            current_safety_deadline_ms,
        }
    }

    pub(crate) const fn deadlines(self) -> Option<([u64; 3], Option<u64>)> {
        match self {
            Self::Unavailable => None,
            Self::Learned {
                play_start_p50_ms,
                play_start_p95_ms,
                play_start_p99_ms,
                current_safety_deadline_ms,
                ..
            } => Some((
                [play_start_p50_ms, play_start_p95_ms, play_start_p99_ms],
                current_safety_deadline_ms,
            )),
        }
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "watch/test_support.rs"]
mod test_support;

const fn clamp_bps(value: u16) -> u16 {
    if value > 10_000 {
        10_000
    } else {
        value
    }
}

const fn maximum(left: u64, right: u64) -> u64 {
    if left > right {
        left
    } else {
        right
    }
}
