mod capture;
mod range_alias_policy;
mod reserve_progress_policy;
mod run;

use range_alias_policy::{is_legacy_range_alias, RecordedRangeAliasGenerationPolicy};
use reserve_progress_policy::{is_legacy_reserve_progress, RecordedReserveProgressPolicy};

use crate::adaptive::{
    AllocationPlan, HlsGenerationPolicy, OriginAdmissionGenerationPolicy, PlannerContext,
    WarpGenerationPolicies,
};
use crate::origin_model::OriginModel;
use serde::{Deserialize, Serialize};

const MODEL_BYTE_LIMIT: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordedPlannerReplayCapsule {
    complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base: Option<AllocationPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    context: Option<PlannerContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    origins: Option<OriginModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    config: Option<RecordedPlannerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    controller_prices: Option<super::RecordedResourcePrices>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    network: Option<RecordedNetworkState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    price_epoch: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_feedback: Option<crate::adaptive::ResourceFeedback>,
    #[serde(default, skip_serializing_if = "is_legacy_hls_generation")]
    hls_generation_policy: RecordedHlsGenerationPolicy,
    #[serde(default, skip_serializing_if = "is_legacy_promotion_generation")]
    promotion_generation_policy: RecordedPromotionGenerationPolicy,
    #[serde(default, skip_serializing_if = "is_legacy_range_alias")]
    range_alias_generation_policy: RecordedRangeAliasGenerationPolicy,
    #[serde(default, skip_serializing_if = "is_legacy_origin_admission_generation")]
    origin_admission_generation_policy: RecordedOriginAdmissionGenerationPolicy,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordedHlsGenerationPolicy {
    #[default]
    LegacyWholeStage,
    BoundedObjectCursor,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordedPromotionGenerationPolicy {
    #[default]
    LegacyLatentGrant,
    ObservedResponse,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordedOriginAdmissionGenerationPolicy {
    #[default]
    LegacyUnclassified,
    TypedIntent,
}

impl From<HlsGenerationPolicy> for RecordedHlsGenerationPolicy {
    fn from(value: HlsGenerationPolicy) -> Self {
        match value {
            HlsGenerationPolicy::LegacyWholeStage => Self::LegacyWholeStage,
            HlsGenerationPolicy::BoundedObjectCursor => Self::BoundedObjectCursor,
        }
    }
}

impl RecordedHlsGenerationPolicy {
    const fn restore(self) -> HlsGenerationPolicy {
        match self {
            Self::LegacyWholeStage => HlsGenerationPolicy::LegacyWholeStage,
            Self::BoundedObjectCursor => HlsGenerationPolicy::BoundedObjectCursor,
        }
    }
}

impl From<crate::adaptive::PromotionGenerationPolicy> for RecordedPromotionGenerationPolicy {
    fn from(value: crate::adaptive::PromotionGenerationPolicy) -> Self {
        match value {
            crate::adaptive::PromotionGenerationPolicy::LegacyLatentGrant => {
                Self::LegacyLatentGrant
            }
            crate::adaptive::PromotionGenerationPolicy::ObservedResponse => Self::ObservedResponse,
        }
    }
}

impl RecordedPromotionGenerationPolicy {
    const fn restore(self) -> crate::adaptive::PromotionGenerationPolicy {
        match self {
            Self::LegacyLatentGrant => {
                crate::adaptive::PromotionGenerationPolicy::LegacyLatentGrant
            }
            Self::ObservedResponse => crate::adaptive::PromotionGenerationPolicy::ObservedResponse,
        }
    }
}

impl From<OriginAdmissionGenerationPolicy> for RecordedOriginAdmissionGenerationPolicy {
    fn from(value: OriginAdmissionGenerationPolicy) -> Self {
        match value {
            OriginAdmissionGenerationPolicy::LegacyUnclassified => Self::LegacyUnclassified,
            OriginAdmissionGenerationPolicy::TypedIntent => Self::TypedIntent,
        }
    }
}

impl RecordedOriginAdmissionGenerationPolicy {
    const fn restore(self) -> OriginAdmissionGenerationPolicy {
        match self {
            Self::LegacyUnclassified => OriginAdmissionGenerationPolicy::LegacyUnclassified,
            Self::TypedIntent => OriginAdmissionGenerationPolicy::TypedIntent,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct RecordedPlannerConfig {
    beam_depth: u64,
    beam_width: u64,
    beam_expansions: u64,
    beam_latency_us: u64,
    twin_particles: u16,
    twin_tail_bps: u16,
    semantic_top_k: u64,
    semantic_epsilon_micros: u64,
    safety_rescue_bps: u16,
    emergency_rescue_bps: u16,
    #[serde(default, skip_serializing_if = "is_legacy_reserve_progress")]
    reserve_progress_policy: RecordedReserveProgressPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum RecordedNetworkState {
    Uninitialized,
    Initialized {
        capacity: u64,
        refill_per_second: u64,
        tokens: u64,
        updated_at_ms: u64,
        #[serde(default, skip_serializing_if = "is_zero")]
        refill_milli_bytes: u64,
        #[serde(default, skip_serializing_if = "is_zero")]
        debt_bytes: u64,
    },
}

const fn is_zero(value: &u64) -> bool {
    *value == 0
}

fn is_legacy_hls_generation(value: &RecordedHlsGenerationPolicy) -> bool {
    *value == RecordedHlsGenerationPolicy::LegacyWholeStage
}

fn is_legacy_promotion_generation(value: &RecordedPromotionGenerationPolicy) -> bool {
    *value == RecordedPromotionGenerationPolicy::LegacyLatentGrant
}

fn is_legacy_origin_admission_generation(value: &RecordedOriginAdmissionGenerationPolicy) -> bool {
    *value == RecordedOriginAdmissionGenerationPolicy::LegacyUnclassified
}

impl RecordedPlannerReplayCapsule {
    fn generation_policies(&self) -> WarpGenerationPolicies {
        WarpGenerationPolicies {
            hls: self.hls_generation_policy.restore(),
            promotion: self.promotion_generation_policy.restore(),
            range_alias: self.range_alias_generation_policy.restore(),
            origin_admission: self.origin_admission_generation_policy.restore(),
        }
    }
}

pub(in crate::adaptive::decision) use capture::capture;
pub(in crate::adaptive::decision) use run::verify;
