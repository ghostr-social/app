use super::{FeedOffset, PromotionOpportunity, RetrievalRequest};
use crate::media_timeline::StartupFootprint;
use crate::playback::{EstimateConfidence, PlaybackPhase};
use crate::{ActionId, ByteRange, PostId};

mod hls;
#[cfg(test)]
mod in_flight_action_api_test;
mod replay;
pub use hls::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewProbability(f64);

impl ViewProbability {
    /// Creates a bounded probability.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotError::InvalidProbability`] for non-finite values or values outside
    /// the inclusive range from zero to one.
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
    pub authority: CurrentAuthority,
    pub phase: PlaybackPhase,
    pub buffer_ahead_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentAuthority {
    Provisional,
    Canonical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkSnapshot {
    pub throughput_bps: u64,
    pub rtt_ms: u64,
    pub packet_loss_bps: u16,
    pub connection_capacity: usize,
    pub connection_ceiling: usize,
    pub per_authority_request_limit: usize,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlayerPreparation {
    #[default]
    Unverified,
    Initializing,
    PluginReady,
    FirstFrameRendered,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayableRange {
    pub bytes: ByteRange,
    pub playable_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InFlightAction {
    pub action_id: ActionId,
    pub request: RetrievalRequest,
    pub effective_bytes: ByteRange,
    pub reserved_storage_bytes: u64,
    pub promotion_opportunity: Option<PromotionOpportunity>,
    pub source: String,
    pub committed_until_ms: u64,
    pub identity_current: bool,
    pub cancelling: bool,
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
    pub feed_offset: FeedOffset,
    pub view_probability: ViewProbability,
    /// False for cache-only inventory outside the bounded retrieval window.
    pub retrieval_eligible: bool,
    pub total_bytes: Option<u64>,
    pub bitrate_bps: u64,
    pub duration_ms: u64,
    pub layout: MediaLayout,
    pub preferred_source: Option<String>,
    pub startup: Option<StartupFootprint>,
    pub player_preparation: PlayerPreparation,
    pub direct_playback_blocked: bool,
    pub(crate) timeline_probe: Option<PlayableRange>,
    pub playable_ranges: Vec<PlayableRange>,
    /// Bytes a live consumer is blocked on right now (a gateway read
    /// outside the buffered region). Always fetched, independent of
    /// how comfortable the playback reserve currently is.
    pub demanded: Option<ByteRange>,
    pub(crate) present: Vec<ByteRange>,
    pub finalized: bool,
    pub(crate) recently_evicted: Vec<ByteRange>,
    pub in_flight: Vec<InFlightAction>,
    pub origins: Vec<OriginHealth>,
    pub evidence: crate::evidence::EvidenceAssessment,
}

impl CandidateSnapshot {
    pub(super) fn needs_bootstrap(&self) -> bool {
        !self.evidence.size.reliable
            || self.total_bytes.is_none()
            || self.layout == MediaLayout::Unknown
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayabilitySnapshot {
    pub observed_at_ms: u64,
    pub commitment_ms: u64,
    /// Upper bound for one origin range request; missing extents are
    /// merged and split so no request exceeds it.
    pub request_slice_bytes: u64,
    pub playback: PlaybackSnapshot,
    pub network: NetworkSnapshot,
    pub storage: StorageSnapshot,
    pub navigation: NavigationSnapshot,
    pub candidates: Vec<CandidateSnapshot>,
    pub hls_candidates: Vec<HlsCandidateSnapshot>,
}
