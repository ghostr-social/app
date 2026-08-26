use super::FetchFailure;
use crate::segmented::fetch::failure_policy::{FailureDisposition, FailurePolicy};
use core::fmt::{Display, Formatter};
use ghostr_engine::adaptive::ResourceCost;
use ghostr_engine::origin_model::ErrorReason;

impl FetchFailure {
    pub(in crate::segmented) const fn origin(&self) -> Option<super::OriginTelemetry> {
        self.origin
    }

    pub(in crate::segmented) const fn reason(&self) -> ErrorReason {
        self.reason
    }

    pub(in crate::segmented) const fn network_bytes(&self) -> u64 {
        self.network_bytes
    }

    pub(in crate::segmented) const fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub(in crate::segmented) const fn is_superseded(&self) -> bool {
        self.superseded
    }

    pub(in crate::segmented) const fn response_completed(&self) -> bool {
        self.response_completed
    }

    pub(crate) fn disposition(&self) -> FailureDisposition {
        self.effective_policy().disposition
    }

    pub(in crate::segmented) fn records_origin_evidence(&self) -> bool {
        self.effective_policy().records_origin_evidence()
    }

    pub(in crate::segmented) fn actual_resources(&self) -> Option<ResourceCost> {
        self.admitted
            .then(|| ResourceCost::new(self.network_bytes, 0, 0, 1))
    }

    fn effective_policy(&self) -> FailurePolicy {
        self.status
            .map(FailurePolicy::for_status)
            .unwrap_or(self.policy)
    }
}

impl Display for FetchFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.error, formatter)
    }
}

#[cfg(test)]
#[path = "evidence_axiom_test.rs"]
pub(crate) mod axiom_test_support;
