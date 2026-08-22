mod capture;
mod run;

use crate::adaptive::HlsGenerationPolicy;
use crate::adaptive::{AllocationPlan, PlannerContext};
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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RecordedHlsGenerationPolicy {
    #[default]
    LegacyWholeStage,
    BoundedObjectCursor,
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

pub(in crate::adaptive::decision) use capture::capture;
pub(in crate::adaptive::decision) use run::verify;
