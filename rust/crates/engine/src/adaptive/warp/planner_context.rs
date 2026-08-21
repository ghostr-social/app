use super::{
    HedgeInput, IdentityProof, RequestOccupancy, ResourceObservation, SemanticScore, TwinEpochs,
};
use crate::adaptive::{EpsilonBuckets, PlayabilitySnapshot};
use crate::{ActionId, PostId};
use std::collections::BTreeMap;

mod candidate;
pub use candidate::{
    HeadProbeHistory, PlannerCandidateContext, PlannerCapability, PlannerQuality,
    PreviewAvailability, TransformCapability,
};

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
    request_occupancy: RequestOccupancy,
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
                        head_probe: HeadProbeHistory::Unobserved,
                    },
                )
            })
            .collect();
        Self {
            candidates,
            active: BTreeMap::new(),
            limits: default_limits(snapshot),
            request_occupancy: RequestOccupancy::from_sources(
                snapshot
                    .candidates
                    .iter()
                    .flat_map(|candidate| &candidate.in_flight)
                    .map(|active| active.source.as_str()),
            ),
            feedback: None,
            epochs: TwinEpochs::new(0, 0, 0),
            epsilon: EpsilonBuckets::new(20, snapshot.request_slice_bytes, 100, 100),
        }
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

    pub fn with_request_occupancy(mut self, occupancy: RequestOccupancy) -> Self {
        self.request_occupancy = occupancy;
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

    pub(super) fn request_occupancy(&self) -> &RequestOccupancy {
        &self.request_occupancy
    }

    pub(super) fn remaining_request_slots(&self) -> u16 {
        self.limits
            .request_tokens
            .saturating_sub(self.request_occupancy.total().min(u16::MAX as usize) as u16)
    }
}

fn default_limits(snapshot: &PlayabilitySnapshot) -> PlannerLimits {
    let rate = snapshot.network.throughput_bps / 8;
    PlannerLimits {
        network_burst_bytes: rate.saturating_mul(2).max(snapshot.request_slice_bytes),
        network_rate_bytes_per_second: rate.max(1),
        cpu_ms: 0,
        request_tokens: snapshot.network.connection_capacity.min(u16::MAX as usize) as u16,
        per_origin_requests: snapshot
            .network
            .per_authority_request_limit
            .min(u16::MAX as usize) as u16,
    }
}
