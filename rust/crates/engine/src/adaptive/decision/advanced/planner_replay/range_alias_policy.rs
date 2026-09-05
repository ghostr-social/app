use crate::adaptive::RangeAliasGenerationPolicy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RecordedRangeAliasGenerationPolicy {
    #[default]
    LegacyIndependentActions,
    PromotableDominance,
}

impl From<RangeAliasGenerationPolicy> for RecordedRangeAliasGenerationPolicy {
    fn from(value: RangeAliasGenerationPolicy) -> Self {
        match value {
            RangeAliasGenerationPolicy::LegacyIndependentActions => Self::LegacyIndependentActions,
            RangeAliasGenerationPolicy::PromotableDominance => Self::PromotableDominance,
        }
    }
}

pub(super) fn is_legacy_range_alias(value: &RecordedRangeAliasGenerationPolicy) -> bool {
    *value == RecordedRangeAliasGenerationPolicy::LegacyIndependentActions
}
