use crate::playback::{BufferTarget, PlaybackObservation, PlaybackPhase};

pub(super) const FULL_CAPACITY_MILLI: u64 = 1_000;
// Reserve capacity while refilling so the buffer converges instead of merely
// staying flat at its risky level.
pub(super) const RECOVERY_CAPACITY_MILLI: u64 = 850;
pub(super) const CRITICAL_CAPACITY_MILLI: u64 = 650;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BufferRisk {
    Frozen,
    Critical,
    Recovering,
    Comfortable,
}

impl BufferRisk {
    pub(super) fn from(observation: PlaybackObservation, target: BufferTarget) -> Self {
        match observation.phase() {
            PlaybackPhase::Paused | PlaybackPhase::Ended | PlaybackPhase::Inactive => Self::Frozen,
            PlaybackPhase::Starting | PlaybackPhase::NetworkStalled => Self::Critical,
            PlaybackPhase::Playing => playing_risk(observation, target),
        }
    }

    pub(super) fn capacity_milli(self) -> u64 {
        match self {
            Self::Frozen | Self::Comfortable => FULL_CAPACITY_MILLI,
            Self::Recovering => RECOVERY_CAPACITY_MILLI,
            Self::Critical => CRITICAL_CAPACITY_MILLI,
        }
    }
}

fn playing_risk(observation: PlaybackObservation, target: BufferTarget) -> BufferRisk {
    let buffered = observation.buffer_ahead();
    if buffered <= target.emergency_horizon() {
        return BufferRisk::Critical;
    }
    if buffered < target.steady() {
        return BufferRisk::Recovering;
    }
    BufferRisk::Comfortable
}
