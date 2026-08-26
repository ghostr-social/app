use super::super::super::privacy::DecisionPrivacy;
use super::request::{self, RecordedRetrievalRequest};
use crate::adaptive::{Allocation, AllocationReason, CandidateUtility, PreemptionAuthority};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedTransfer {
    pub(crate) post_id: String,
    pub(crate) source_id: String,
    pub request: RecordedRetrievalRequest,
    pub(crate) expected_playable_gain_ms: u64,
    pub(crate) utility: RecordedCandidateUtility,
    pub authority: RecordedPreemptionAuthority,
    pub(crate) commitment_until_ms: u64,
    pub reason: RecordedAllocationReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedCandidateUtility {
    pub(crate) view_probability_bits: u64,
    pub(crate) additional_playable_ms: u64,
    pub(crate) expected_delivery_ms: u64,
    pub(crate) score_bits: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedPreemptionAuthority {
    PlaybackCritical,
    Transition,
    Speculative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedAllocationReason {
    CurrentStallPrevention,
    CurrentBufferReserve,
    LikelyNextTransition,
    RapidNavigationCoverage,
    MediaBootstrap,
    MediaLayoutDiscovery,
    NextStartability,
    UsefulCommitment,
}

pub(super) fn capture(value: &Allocation, privacy: &DecisionPrivacy) -> RecordedTransfer {
    RecordedTransfer {
        post_id: privacy.post(value.post.as_str()),
        source_id: privacy.source(&value.source),
        request: request::capture(value.request),
        expected_playable_gain_ms: value.expected_playable_gain_ms,
        utility: value.utility.into(),
        authority: value.authority.into(),
        commitment_until_ms: value.commitment_until_ms,
        reason: value.reason.into(),
    }
}

impl RecordedTransfer {
    pub(super) fn projection(&self, name: &'static str) -> (&'static str, &str, u64, u64) {
        let (start, end) = self.request.bytes();
        (name, &self.source_id, start, end)
    }
}

impl From<CandidateUtility> for RecordedCandidateUtility {
    fn from(value: CandidateUtility) -> Self {
        Self {
            view_probability_bits: value.view_probability.to_bits(),
            additional_playable_ms: value.additional_playable_ms,
            expected_delivery_ms: value.expected_delivery_ms,
            score_bits: value.score.to_bits(),
        }
    }
}

impl From<PreemptionAuthority> for RecordedPreemptionAuthority {
    fn from(value: PreemptionAuthority) -> Self {
        match value {
            PreemptionAuthority::PlaybackCritical => Self::PlaybackCritical,
            PreemptionAuthority::Transition => Self::Transition,
            PreemptionAuthority::Speculative => Self::Speculative,
        }
    }
}

impl From<AllocationReason> for RecordedAllocationReason {
    fn from(value: AllocationReason) -> Self {
        match value {
            AllocationReason::CurrentStallPrevention
            | AllocationReason::CurrentBufferReserve
            | AllocationReason::LikelyNextTransition
            | AllocationReason::RapidNavigationCoverage => playback_reason(value),
            AllocationReason::MediaBootstrap
            | AllocationReason::MediaLayoutDiscovery
            | AllocationReason::NextStartability
            | AllocationReason::UsefulCommitment => preparation_reason(value),
        }
    }
}

fn playback_reason(value: AllocationReason) -> RecordedAllocationReason {
    match value {
        AllocationReason::CurrentStallPrevention => {
            RecordedAllocationReason::CurrentStallPrevention
        }
        AllocationReason::CurrentBufferReserve => RecordedAllocationReason::CurrentBufferReserve,
        AllocationReason::LikelyNextTransition => RecordedAllocationReason::LikelyNextTransition,
        AllocationReason::RapidNavigationCoverage => {
            RecordedAllocationReason::RapidNavigationCoverage
        }
        _ => unreachable!("only playback reasons are routed here"),
    }
}

fn preparation_reason(value: AllocationReason) -> RecordedAllocationReason {
    match value {
        AllocationReason::MediaBootstrap => RecordedAllocationReason::MediaBootstrap,
        AllocationReason::MediaLayoutDiscovery => RecordedAllocationReason::MediaLayoutDiscovery,
        AllocationReason::NextStartability => RecordedAllocationReason::NextStartability,
        AllocationReason::UsefulCommitment => RecordedAllocationReason::UsefulCommitment,
        _ => unreachable!("only preparation reasons are routed here"),
    }
}
