use super::{RequestOccupancy, ResourceObservation, SemanticScore, TwinEpochs};
use crate::adaptive::{EpsilonBuckets, PlayabilitySnapshot};
use crate::{ActionId, PostId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod active;
mod candidate;
mod replay;
mod request_scope;
mod watch;
pub use active::ActivePlannerContext;
pub use candidate::{
    HeadProbeHistory, PlannerCandidateContext, PlannerCapability, PlannerQuality,
    PlannerRetryAvailability, PreviewAvailability, TransformCapability,
    INLINE_BLURHASH_PREVIEW_QUALITY_MICROS,
};
pub use request_scope::SoftRequestCommitment;
pub use watch::PlannerWatchEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannerRetryEvidence {
    pub post: PostId,
    pub availability: PlannerRetryAvailability,
}

impl PlannerRetryEvidence {
    pub const fn new(post: PostId, availability: PlannerRetryAvailability) -> Self {
        Self { post, availability }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlannerLimits {
    pub network_burst_bytes: u64,
    pub network_rate_bytes_per_second: u64,
    pub cpu_ms: u64,
    pub request_tokens: u16,
    pub per_origin_requests: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceFeedback {
    pub actual: ResourceObservation,
    pub target: ResourceObservation,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PlannerContext {
    candidates: BTreeMap<PostId, PlannerCandidateContext>,
    active: BTreeMap<ActionId, ActivePlannerContext>,
    pub limits: PlannerLimits,
    request_occupancy: RequestOccupancy,
    request_scope: Option<request_scope::RequestScope>,
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
                        watch: PlannerWatchEvidence::Unavailable,
                        head_probe: HeadProbeHistory::Unobserved,
                        retry: PlannerRetryAvailability::Ready,
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
            request_scope: None,
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

    pub fn with_soft_request_capacity(
        mut self,
        ordinary_tokens: u16,
        soft: Vec<SoftRequestCommitment>,
    ) -> Self {
        self.request_scope = Some(request_scope::RequestScope::new(ordinary_tokens, soft));
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

    pub(super) fn active_contexts(&self) -> impl Iterator<Item = &ActivePlannerContext> {
        self.active.values()
    }

    pub(super) fn request_occupancy(&self) -> &RequestOccupancy {
        &self.request_occupancy
    }

    pub(super) fn permits_request(&self, post: &PostId) -> bool {
        self.candidate(post)
            .is_some_and(|candidate| candidate.retry.permits_request())
    }

    pub(super) fn retry_evidence(
        &self,
        snapshot: &PlayabilitySnapshot,
    ) -> Vec<PlannerRetryEvidence> {
        snapshot
            .candidates
            .iter()
            .filter_map(|item| {
                let candidate = self.candidates.get(&item.post)?;
                (candidate.retry != PlannerRetryAvailability::Ready)
                    .then(|| PlannerRetryEvidence::new(item.post.clone(), candidate.retry))
            })
            .collect()
    }

    pub(super) fn remaining_request_slots(&self) -> u16 {
        self.limits
            .request_tokens
            .saturating_sub(self.request_occupancy.total().min(u16::MAX as usize) as u16)
    }

    pub(super) fn request_admits(&self, action: &crate::adaptive::ActionNode) -> bool {
        self.request_scope
            .as_ref()
            .is_none_or(|scope| scope.admits(action, self.request_occupancy.total()))
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
