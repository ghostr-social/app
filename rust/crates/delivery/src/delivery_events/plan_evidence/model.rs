use super::super::DeliveryNetworkStatus;
use crate::startup_certificate::StartupCertificate;
use ghostr_engine::adaptive::AllocationPlan;
use ghostr_engine::origin_model::NetworkClass;
use ghostr_engine::PostId;

#[derive(Clone, Debug, PartialEq)]
pub struct PlanEvidence {
    pub revision: u64,
    pub observed_at_ms: u64,
    pub current: Option<PostId>,
    pub focus_generation: Option<u64>,
    pub focus_covers_from: Option<u64>,
    pub network_status_generation: u64,
    pub network_class: NetworkClass,
    pub network_profile_generation: u64,
    pub plan: AllocationPlan,
    pub startups: Vec<StartupCertificate>,
}

pub(crate) struct PlanPublicationContext {
    pub(super) observed_at_ms: u64,
    pub(super) current: Option<PostId>,
    pub(super) focus_generation: Option<u64>,
    pub(super) focus_covers_from: Option<u64>,
    pub(super) network_status: DeliveryNetworkStatus,
    pub(super) network_profile_generation: u64,
}

impl PlanPublicationContext {
    pub(crate) const fn new(observed_at_ms: u64, current: Option<PostId>) -> Self {
        Self {
            observed_at_ms,
            current,
            focus_generation: None,
            focus_covers_from: None,
            network_status: DeliveryNetworkStatus::unavailable(),
            network_profile_generation: 0,
        }
    }

    pub(crate) const fn with_focus(
        mut self,
        generation: Option<u64>,
        covers_from: Option<u64>,
    ) -> Self {
        self.focus_generation = generation;
        self.focus_covers_from = covers_from;
        self
    }

    pub(crate) const fn with_network(
        mut self,
        status: DeliveryNetworkStatus,
        profile_generation: u64,
    ) -> Self {
        self.network_status = status;
        self.network_profile_generation = profile_generation;
        self
    }
}
