//! Playback evidence used by the delivery policy.
//!
//! Player phases are explicit so pause, completion, and inactivity can
//! never be mistaken for a network stall.

mod buffer;
mod network;
mod session;

pub use buffer::{AdaptiveBufferPolicy, BufferTarget};
pub use network::{EstimateConfidence, NetworkConditions};
pub use session::{PlaybackObservationSequence, PlaybackSession, PlaybackStatus};
use std::time::Duration;

/// Byte window shared by protected prefetch grants and progressive demand.
pub const PLAYBACK_SLICE_BYTES: u64 = 256 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackPhase {
    Starting,
    Playing,
    NetworkStalled,
    Paused,
    Ended,
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

    pub fn bits_per_second(self) -> u64 {
        let scaled = u128::from(self.bitrate_bits_per_second)
            .saturating_mul(u128::from(self.playback_rate_milli))
            / 1_000;
        scaled.min(u128::from(u64::MAX)) as u64
    }

    fn media_bitrate(self) -> u64 {
        self.bitrate_bits_per_second
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
        self.buffered_extent - self.position
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

    pub fn needs_urgent_refill(
        self,
        inflow_bits_per_second: u64,
        media: MediaConsumption,
        horizon: Duration,
    ) -> bool {
        match self.phase {
            PlaybackPhase::Starting | PlaybackPhase::NetworkStalled => true,
            PlaybackPhase::Playing => {
                self.buffer_ahead() <= horizon
                    || self
                        .time_to_empty(inflow_bits_per_second, media)
                        .is_some_and(|remaining| remaining <= horizon)
            }
            PlaybackPhase::Paused | PlaybackPhase::Ended | PlaybackPhase::Inactive => false,
        }
    }

    pub fn time_to_empty(
        self,
        inflow_bits_per_second: u64,
        media: MediaConsumption,
    ) -> Option<Duration> {
        let consumption = u128::from(media.bits_per_second());
        let deficit = consumption.checked_sub(u128::from(inflow_bits_per_second))?;
        if deficit == 0 || media.media_bitrate() == 0 {
            return None;
        }
        let buffered_bits_ms = self
            .buffer_ahead()
            .as_millis()
            .saturating_mul(u128::from(media.media_bitrate()));
        let empty_ms = buffered_bits_ms.div_ceil(deficit);
        Some(Duration::from_millis(
            empty_ms.min(u128::from(u64::MAX)) as u64
        ))
    }
}
