use super::QoeTracker;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QoeStats {
    pub(crate) playback_sessions: u64,
    pub(crate) user_navigations: u64,
    pub(crate) first_frames: u64,
    pub(crate) startup_failures: u64,
    pub(crate) startup_total_ms: u64,
    pub(crate) startup_max_ms: u64,
    pub(crate) buffer_samples: u64,
    pub(crate) buffer_ahead_total_ms: u64,
    pub(crate) minimum_buffer_ahead_ms: Option<u64>,
    pub(crate) stall_events: u64,
    pub(crate) stall_total_ms: u64,
    pub(crate) decode_failures: u64,
    pub(crate) abandonments: u64,
    pub(crate) completions: u64,
    pub(crate) transport_substitutions: u64,
    pub(crate) rank_displacement_total: u64,
    pub(crate) rescue_wait_total_ms: u64,
    pub(crate) eta_unavailable_rescues: u64,
    pub(crate) eta_too_long_rescues: u64,
    pub(crate) delivery_failed_rescues: u64,
    pub(crate) grace_expired_rescues: u64,
}

impl QoeStats {
    pub(crate) fn startup_eta_ms(&self) -> u64 {
        if self.first_frames == 0 {
            return QoeTracker::DEFAULT_STARTUP_ETA_MS;
        }
        let mean = self.startup_total_ms / self.first_frames;
        mean.saturating_add(self.startup_max_ms.saturating_sub(mean) / 2)
            .clamp(50, 5_000)
    }
}
