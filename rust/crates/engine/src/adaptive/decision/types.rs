use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShadowPrices {
    pub network_micros: u64,
    pub storage_time_micros: u64,
    pub cpu_micros: u64,
    pub request_micros: u64,
}

impl ShadowPrices {
    pub const fn new(network: u64, storage: u64, cpu: u64, requests: u64) -> Self {
        Self {
            network_micros: network,
            storage_time_micros: storage,
            cpu_micros: cpu,
            request_micros: requests,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionModelInput {
    pub source: String,
    /// mean, lower, upper, selected in basis points.
    pub success_bps: [u16; 4],
    /// mean, lower, upper, selected in basis points.
    pub range_compliance_bps: [u16; 4],
    /// p10, p50, p90, p95, p99, selected.
    pub ttfb_ms: [u64; 6],
    /// p10, p50, p90, p95, p99, selected.
    pub throughput_bps: [u64; 6],
    pub effective_samples: u64,
    pub adapting: bool,
    pub uncertainty_bps: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProbabilityEstimateRecord {
    pub mean_bps: u16,
    pub lower_bps: u16,
    pub upper_bps: u16,
    pub selected_bps: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuantileRecord {
    pub p10: u64,
    pub p50: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub selected: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelQuantiles {
    pub(super) source_id: String,
    pub(super) success: ProbabilityEstimateRecord,
    pub(super) range_compliance: ProbabilityEstimateRecord,
    pub(crate) ttfb_ms: QuantileRecord,
    pub(super) throughput_bps: QuantileRecord,
    pub(super) effective_samples: u64,
    pub(super) adapting: bool,
    pub(super) uncertainty_bps: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionAction {
    pub post_id: String,
    pub source_id: String,
    pub request: String,
    pub bytes_start: u64,
    pub bytes_end: u64,
    pub expected_playable_gain_ms: u64,
    pub utility_micros: i64,
    pub reason: String,
    pub retained: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrunedReason {
    RetrievalIneligible,
    NoAvailableOrigin,
    AlreadyReady,
    NoStorageCapacity,
    NoTransferCapacity,
    LowerUtility,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PrunedCandidate {
    pub(super) post_id: String,
    pub(crate) reasons: Vec<PrunedReason>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DecisionOutcome {
    Pending,
    Succeeded {
        bytes: u64,
        elapsed_ms: u64,
    },
    HeadObserved {
        content_length: u64,
        accept_ranges: Option<bool>,
        elapsed_ms: u64,
    },
    ClaimRefused {
        reason: ProbeClaimRefusal,
    },
    Failed {
        class: String,
        elapsed_ms: u64,
    },
    Cancelled {
        bytes: u64,
        elapsed_ms: u64,
    },
    Superseded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeClaimRefusal {
    PoolAtCapacity,
    AlreadyProbing,
    AlreadyProbed,
    DeferredToBody,
    RetryCooling,
    CandidateMissing,
    SourceNotOffered,
    SourceRetired,
    EvidenceComplete,
    IdentityMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionReplayStatus {
    Verified,
    UnsupportedSchema,
    StateHashMismatch,
    PlanMismatch,
    AdvancedReplayUnavailable,
}
