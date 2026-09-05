//! Playback evidence used by the delivery policy.
//!
//! Player phases are explicit so pause, completion, and inactivity can
//! never be mistaken for a network stall.

mod buffer;
mod continuation;
mod deficit;
mod indexed;
pub use indexed::ContinuationConditions;
mod network;
mod session;

pub use buffer::{AdaptiveBufferPolicy, BufferTarget};
use core::time::Duration;
pub(crate) use deficit::{BufferScenario, UsableArrival};
pub use network::{EstimateConfidence, NetworkConditions};
pub use session::{PlaybackObservationSequence, PlaybackSession, PlaybackStatus};

/// Byte window shared by protected prefetch grants and progressive demand.
pub const PLAYBACK_SLICE_BYTES: u64 = 256 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackPhase {
    Starting,
    Playing,
    NetworkStalled,
    Paused,
    Ended,
    Failed,
    Inactive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackObservationError {
    BufferedBeforePosition,
    ZeroPlaybackRate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaConsumption {
    bitrate_bits_per_second: u64,
    playback_rate_milli: u16,
}

impl MediaConsumption {
    pub const fn new(bitrate_bits_per_second: u64, playback_rate_milli: u16) -> Self {
        Self {
            bitrate_bits_per_second,
            playback_rate_milli,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaybackObservation {
    position: Duration,
    buffered_extent: Duration,
    playback_rate_milli: u16,
    phase: PlaybackPhase,
}

impl PlaybackObservation {
    /// Creates a coherent playback observation.
    ///
    /// # Errors
    ///
    /// Returns an error when the buffered extent precedes the position or the playback rate is
    /// zero.
    pub fn try_new(
        position: Duration,
        buffered_extent: Duration,
        playback_rate_milli: u16,
        phase: PlaybackPhase,
    ) -> Result<Self, PlaybackObservationError> {
        if buffered_extent < position {
            return Err(PlaybackObservationError::BufferedBeforePosition);
        }
        if playback_rate_milli == 0 {
            return Err(PlaybackObservationError::ZeroPlaybackRate);
        }
        Ok(Self {
            position,
            buffered_extent,
            playback_rate_milli,
            phase,
        })
    }

    pub fn buffer_ahead(self) -> Duration {
        self.buffered_extent.saturating_sub(self.position)
    }

    pub fn position(self) -> Duration {
        self.position
    }

    pub fn playback_rate_milli(self) -> u16 {
        self.playback_rate_milli
    }

    pub fn phase(self) -> PlaybackPhase {
        self.phase
    }
}
