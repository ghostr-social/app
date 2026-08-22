use super::{RequestOccupancy, SemanticScore, TwinEpochs};
use crate::adaptive::{EpsilonBuckets, PlayabilitySnapshot};
use crate::origin_model::NetworkClass;
use crate::{ActionId, PostId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod active;
mod candidate;
#[cfg(test)]
#[path = "planner_context/hls_request_scope_test.rs"]
mod hls_request_scope_test;
mod replay;
mod request_scope;
mod resource_feedback;
mod retry;
mod segmented_storage;
mod unavailable;
mod watch;
pub use active::ActivePlannerContext;
pub use candidate::{
    HeadProbeHistory, PlannerCandidateContext, PlannerCapability, PlannerQuality,
    PlannerRetryAvailability, PreviewAvailability, TransformCapability,
    INLINE_BLURHASH_PREVIEW_QUALITY_MICROS,
};
pub use request_scope::SoftRequestCommitment;
pub use resource_feedback::{ResourceFeedback, ResourceFeedbackCursor, ResourcePriceSnapshot};
pub use segmented_storage::SegmentedStorageBudget;
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PlannerContext {
    candidates: BTreeMap<PostId, PlannerCandidateContext>,
    active: BTreeMap<ActionId, ActivePlannerContext>,
    #[serde(
        default = "unavailable_network",
        skip_serializing_if = "network_unavailable"
    )]
    network_class: NetworkClass,
    #[serde(default, skip_serializing_if = "SegmentedStorageBudget::is_empty")]
    segmented_storage_available_bytes: SegmentedStorageBudget,
    pub limits: PlannerLimits,
    request_occupancy: RequestOccupancy,
    request_scope: Option<request_scope::RequestScope>,
    pub feedback: Option<ResourceFeedback>,
    pub epochs: TwinEpochs,
    pub epsilon: EpsilonBuckets,
}

impl PlannerContext {
    pub fn explicitly_unavailable(snapshot: &PlayabilitySnapshot) -> Self {
        unavailable::build(snapshot)
    }

    pub fn with_active(mut self, active: ActivePlannerContext) -> Self {
        self.active.insert(active.action, active);
        self
    }

    pub fn with_network_class(mut self, network_class: NetworkClass) -> Self {
        self.network_class = network_class;
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
        hls_tokens: u16,
        soft: Vec<SoftRequestCommitment>,
    ) -> Self {
        self.request_scope = Some(request_scope::RequestScope::new(
            ordinary_tokens,
            hls_tokens,
            soft,
        ));
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

    pub(super) const fn network_class(&self) -> NetworkClass {
        self.network_class
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

const fn unavailable_network() -> NetworkClass {
    NetworkClass::Unavailable
}

fn network_unavailable(value: &NetworkClass) -> bool {
    *value == NetworkClass::Unavailable
}

pub(super) fn default_limits(snapshot: &PlayabilitySnapshot) -> PlannerLimits {
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
