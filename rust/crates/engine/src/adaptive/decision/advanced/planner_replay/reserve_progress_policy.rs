use crate::adaptive::warp::ReserveProgressPolicy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RecordedReserveProgressPolicy {
    #[default]
    LegacyCoverage,
    OrderedReadiness,
}

impl From<ReserveProgressPolicy> for RecordedReserveProgressPolicy {
    fn from(value: ReserveProgressPolicy) -> Self {
        match value {
            ReserveProgressPolicy::LegacyCoverage => Self::LegacyCoverage,
            ReserveProgressPolicy::OrderedReadiness => Self::OrderedReadiness,
        }
    }
}

impl RecordedReserveProgressPolicy {
    pub(super) const fn restore(self) -> ReserveProgressPolicy {
        match self {
            Self::LegacyCoverage => ReserveProgressPolicy::LegacyCoverage,
            Self::OrderedReadiness => ReserveProgressPolicy::OrderedReadiness,
        }
    }
}

pub(super) fn is_legacy_reserve_progress(value: &RecordedReserveProgressPolicy) -> bool {
    *value == RecordedReserveProgressPolicy::LegacyCoverage
}
