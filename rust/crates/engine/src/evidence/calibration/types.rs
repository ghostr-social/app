use super::EvidenceField;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalibrationDimensions {
    pub(super) issuer: Option<String>,
    pub(super) client: Option<String>,
    pub(super) origin: Option<String>,
    pub(super) url: Option<String>,
}

impl CalibrationDimensions {
    pub(crate) fn provider(
        issuer: Option<String>,
        client: Option<String>,
        origin: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            issuer,
            client,
            origin,
            url,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalibrationContext {
    pub(super) dimensions: CalibrationDimensions,
    pub(super) field: EvidenceField,
    pub(super) context: String,
}

impl CalibrationContext {
    pub fn new(
        dimensions: CalibrationDimensions,
        field: EvidenceField,
        context: impl Into<String>,
    ) -> Self {
        Self {
            dimensions,
            field,
            context: context.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalibrationLabel {
    pub(super) context: CalibrationContext,
    pub(super) correct: bool,
    pub(super) observed_at_ms: u64,
    #[serde(default = "full_weight")]
    pub(super) weight_bps: u16,
}

impl CalibrationLabel {
    pub(crate) fn discounted(
        context: CalibrationContext,
        correct: bool,
        observed_at_ms: u64,
        weight_bps: u16,
    ) -> Self {
        Self {
            context,
            correct,
            observed_at_ms,
            weight_bps: weight_bps.min(10_000),
        }
    }
}

#[cfg(any(test, feature = "test"))]
#[path = "types/test_support.rs"]
mod test_support;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReliabilityEstimate {
    pub(crate) mean_bps: u16,
    pub(crate) lower_bound_bps: u16,
    pub(crate) effective_samples_bps: u32,
}

const fn full_weight() -> u16 {
    10_000
}
