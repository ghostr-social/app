use super::{
    HedgeInput, IdentityProof, ResourceObservation, SemanticScore, TransformKind, TwinEpochs,
};
use crate::adaptive::{EpsilonBuckets, PlayabilitySnapshot};
use crate::{ActionId, PostId};
use std::collections::BTreeMap;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HedgeContext {
    input: HedgeInput,
    identity: IdentityProof,
    alternate: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivePlannerContext {
    pub action: ActionId,
    pub continuation_advantage_micros: Option<i64>,
    hedge: Option<HedgeContext>,
}

impl ActivePlannerContext {
    pub const fn new(action: ActionId) -> Self {
        Self {
            action,
            continuation_advantage_micros: None,
            hedge: None,
        }
    }

    pub const fn with_continuation_advantage(mut self, value_micros: i64) -> Self {
        self.continuation_advantage_micros = Some(value_micros);
        self
    }

    pub fn with_hedge(
        mut self,
        input: HedgeInput,
        identity: IdentityProof,
        alternate: impl Into<String>,
    ) -> Self {
        self.hedge = Some(HedgeContext {
            input,
            identity,
            alternate: alternate.into(),
        });
        self
    }

    pub(super) fn hedge(&self) -> Option<(&HedgeInput, IdentityProof, &str)> {
        self.hedge
            .as_ref()
            .map(|item| (&item.input, item.identity.clone(), item.alternate.as_str()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlannerLimits {
    pub network_burst_bytes: u64,
    pub network_rate_bytes_per_second: u64,
    pub cpu_ms: u64,
    pub request_tokens: u16,
    pub per_origin_requests: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceFeedback {
    pub actual: ResourceObservation,
    pub target: ResourceObservation,
}

#[derive(Clone, Debug)]
pub struct PlannerContext {
    candidates: BTreeMap<PostId, PlannerCandidateContext>,
    active: BTreeMap<ActionId, ActivePlannerContext>,
    pub limits: PlannerLimits,
    pub feedback: Option<ResourceFeedback>,
    pub epochs: TwinEpochs,
    pub epsilon: EpsilonBuckets,
}

impl PlannerContext {
    pub fn explicitly_unavailable(snapshot: &PlayabilitySnapshot) -> Self {
        let candidates = snapshot
            .candidates
            .iter()
            .enumerate()
            .map(|(rank, item)| {
                (
                    item.post.clone(),
                    PlannerCandidateContext {
                        semantic: SemanticScore::Unavailable { rank },
                        capability: PlannerCapability::Unavailable,
                        quality: PlannerQuality::Unavailable,
                        preview: PreviewAvailability::Unavailable,
                    },
                )
            })
            .collect();
        Self {
            candidates,
            active: BTreeMap::new(),
            limits: default_limits(snapshot),
            feedback: None,
            epochs: TwinEpochs::new(0, 0, 0),
            epsilon: EpsilonBuckets::new(20, snapshot.request_slice_bytes, 100, 100),
        }
    }

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

    pub fn with_active(mut self, active: ActivePlannerContext) -> Self {
        self.active.insert(active.action, active);
        self
    }

    pub fn with_limits(mut self, limits: PlannerLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_feedback(mut self, feedback: ResourceFeedback) -> Self {
        self.feedback = Some(feedback);
        self
    }

    pub fn with_epochs(mut self, epochs: TwinEpochs) -> Self {
        self.epochs = epochs;
        self
    }

    pub fn with_epsilon(mut self, epsilon: EpsilonBuckets) -> Self {
        self.epsilon = epsilon;
        self
    }

    pub(super) fn candidate(&self, post: &PostId) -> Option<PlannerCandidateContext> {
        self.candidates.get(post).copied()
    }

    pub(super) fn active(&self, action: ActionId) -> Option<&ActivePlannerContext> {
        self.active.get(&action)
    }
}

fn default_limits(snapshot: &PlayabilitySnapshot) -> PlannerLimits {
    let rate = snapshot.network.throughput_bps / 8;
    PlannerLimits {
        network_burst_bytes: rate.saturating_mul(2).max(snapshot.request_slice_bytes),
        network_rate_bytes_per_second: rate.max(1),
        cpu_ms: 0,
        request_tokens: snapshot.network.connection_capacity.min(u16::MAX as usize) as u16,
        per_origin_requests: snapshot.network.connection_ceiling.min(u16::MAX as usize) as u16,
    }
}
