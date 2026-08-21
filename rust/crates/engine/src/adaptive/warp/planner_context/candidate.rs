use super::super::{SemanticScore, TransformKind};
use super::PlannerContext;
use crate::PostId;

/// Durable HEAD history for one representation, excluding transient pool occupancy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadProbeHistory {
    Unobserved,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlannerCapability {
    Unavailable,
    Reported {
        playback_supported: bool,
        transform: Option<TransformCapability>,
        evidence_epoch: u64,
    },
}

impl PlannerCapability {
    pub const fn reported(
        playback_supported: bool,
        transform: Option<TransformCapability>,
        evidence_epoch: u64,
    ) -> Self {
        Self::Reported {
            playback_supported,
            transform,
            evidence_epoch,
        }
    }

    pub const fn required_transform(self) -> Option<TransformCapability> {
        match self {
            Self::Reported {
                playback_supported: false,
                transform,
                ..
            } => transform,
            _ => None,
        }
    }

    pub const fn blocks_direct_playback(self) -> bool {
        matches!(
            self,
            Self::Reported {
                playback_supported: false,
                ..
            }
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformCapability {
    pub kind: TransformKind,
    pub estimated_cpu_ms: u64,
    pub output_upper_bytes: u64,
}

impl TransformCapability {
    pub const fn new(kind: TransformKind, estimated_cpu_ms: u64, output_upper_bytes: u64) -> Self {
        Self {
            kind,
            estimated_cpu_ms,
            output_upper_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlannerQuality {
    Unavailable,
    Estimated {
        expected_micros: u64,
        lower_micros: u64,
        uncertainty_bps: u16,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewAvailability {
    Unavailable,
    Ready { bytes: u64, quality_micros: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlannerCandidateContext {
    pub semantic: SemanticScore,
    pub capability: PlannerCapability,
    pub quality: PlannerQuality,
    pub preview: PreviewAvailability,
    pub head_probe: HeadProbeHistory,
}

impl PlannerContext {
    pub fn with_capability(mut self, post: PostId, capability: PlannerCapability) -> Self {
        if let Some(candidate) = self.candidates.get_mut(&post) {
            candidate.capability = capability;
        }
        self
    }

    pub fn with_semantic(mut self, post: PostId, semantic: SemanticScore) -> Self {
        if let Some(candidate) = self.candidates.get_mut(&post) {
            candidate.semantic = semantic;
        }
        self
    }

    pub fn with_quality(mut self, post: PostId, quality: PlannerQuality) -> Self {
        if let Some(candidate) = self.candidates.get_mut(&post) {
            candidate.quality = quality;
        }
        self
    }

    pub fn with_preview(mut self, post: PostId, preview: PreviewAvailability) -> Self {
        if let Some(candidate) = self.candidates.get_mut(&post) {
            candidate.preview = preview;
        }
        self
    }

    pub fn with_head_probe_history(mut self, post: PostId, history: HeadProbeHistory) -> Self {
        if let Some(candidate) = self.candidates.get_mut(&post) {
            candidate.head_probe = history;
        }
        self
    }
}
