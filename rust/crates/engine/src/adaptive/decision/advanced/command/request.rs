use crate::adaptive::{PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum RecordedRetrievalRequest {
    FetchRange {
        bytes_start: u64,
        bytes_end: u64,
        promotion: Option<RecordedPromotionGrant>,
    },
    FetchWhole {
        contract: RecordedWholeBodyContract,
        reason: RecordedWholeFetchReason,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecordedPromotionGrant {
    pub(crate) maximum_bytes: u64,
    pub(crate) valid_until_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "contract", rename_all = "snake_case")]
pub enum RecordedWholeBodyContract {
    Exact { expected_bytes: u64 },
    Capped { maximum_bytes: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordedWholeFetchReason {
    DirectCrossover,
    PromotedResponse,
    PlannedCompletion,
}

pub(in crate::adaptive::decision) fn capture(value: RetrievalRequest) -> RecordedRetrievalRequest {
    match value {
        RetrievalRequest::FetchRange { bytes, promotion } => RecordedRetrievalRequest::FetchRange {
            bytes_start: bytes.start,
            bytes_end: bytes.end,
            promotion: promotion.map(RecordedPromotionGrant::from),
        },
        RetrievalRequest::FetchWhole { contract, reason } => RecordedRetrievalRequest::FetchWhole {
            contract: recorded_contract(contract),
            reason: recorded_reason(reason),
        },
    }
}

impl RecordedRetrievalRequest {
    pub(in crate::adaptive::decision) fn bytes(self) -> (u64, u64) {
        match self {
            Self::FetchRange {
                bytes_start,
                bytes_end,
                ..
            } => (bytes_start, bytes_end),
            Self::FetchWhole { contract, .. } => (0, contract.maximum_bytes()),
        }
    }
}

impl RecordedWholeBodyContract {
    const fn maximum_bytes(self) -> u64 {
        match self {
            Self::Exact { expected_bytes } => expected_bytes,
            Self::Capped { maximum_bytes } => maximum_bytes,
        }
    }
}

impl From<PromotionGrant> for RecordedPromotionGrant {
    fn from(value: PromotionGrant) -> Self {
        Self {
            maximum_bytes: value.maximum_bytes,
            valid_until_ms: value.valid_until_ms,
        }
    }
}

fn recorded_contract(value: WholeBodyContract) -> RecordedWholeBodyContract {
    match value {
        WholeBodyContract::Exact { expected_bytes } => {
            RecordedWholeBodyContract::Exact { expected_bytes }
        }
        WholeBodyContract::Capped { maximum_bytes } => {
            RecordedWholeBodyContract::Capped { maximum_bytes }
        }
    }
}

fn recorded_reason(value: WholeFetchReason) -> RecordedWholeFetchReason {
    match value {
        WholeFetchReason::DirectCrossover => RecordedWholeFetchReason::DirectCrossover,
        WholeFetchReason::PromotedResponse => RecordedWholeFetchReason::PromotedResponse,
        WholeFetchReason::PlannedCompletion => RecordedWholeFetchReason::PlannedCompletion,
    }
}
