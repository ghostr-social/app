use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::{ByteRange, PostId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewProbability(f64);

impl ViewProbability {
    pub fn new(value: f64) -> Result<Self, SnapshotError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(SnapshotError::InvalidProbability);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    InvalidProbability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaybackSnapshot {
    pub current: PostId,
    pub phase: PlaybackPhase,
    pub buffer_ahead_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkSnapshot {
    pub throughput_bps: u64,
    pub rtt_ms: u64,
    pub packet_loss_bps: u16,
    pub connection_capacity: usize,
    pub connection_ceiling: usize,
    pub confidence: EstimateConfidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageSnapshot {
    pub budget_bytes: u64,
    pub used_bytes: u64,
}

impl StorageSnapshot {
    pub const fn new(budget_bytes: u64, used_bytes: u64) -> Self {
        Self {
            budget_bytes,
            used_bytes,
        }
    }

    pub fn available_bytes(self) -> u64 {
        self.budget_bytes.saturating_sub(self.used_bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavigationSnapshot {
    pub forward_swipes_per_minute: u16,
    pub backward_swipes_per_minute: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaLayout {
    Unknown,
    Streamable,
    RequiresCompleteFile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayableRange {
    pub bytes: ByteRange,
    pub playable_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InFlightRange {
    pub bytes: ByteRange,
    pub source: String,
    pub committed_until_ms: u64,
    pub identity_current: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OriginHealth {
    pub source: String,
    pub available: bool,
    pub throughput_bps: u64,
    pub rtt_ms: u64,
    pub packet_loss_bps: u16,
    pub failure_bps: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateSnapshot {
    pub post: PostId,
    pub feed_distance: usize,
    pub view_probability: ViewProbability,
    pub bitrate_bps: u64,
    pub duration_ms: u64,
    pub layout: MediaLayout,
    pub timeline_probe: Option<PlayableRange>,
    pub playable_ranges: Vec<PlayableRange>,
    pub present: Vec<ByteRange>,
    pub recently_evicted: Vec<ByteRange>,
    pub in_flight: Vec<InFlightRange>,
    pub origins: Vec<OriginHealth>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayabilitySnapshot {
    pub observed_at_ms: u64,
    pub commitment_ms: u64,
    pub playback: PlaybackSnapshot,
    pub network: NetworkSnapshot,
    pub storage: StorageSnapshot,
    pub navigation: NavigationSnapshot,
    pub candidates: Vec<CandidateSnapshot>,
}
