use serde::{Deserialize, Serialize};

use crate::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use crate::media_timeline::{StartupFootprint, StartupProvenance};
use crate::ByteRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct RangeState {
    start: u64,
    end: u64,
}

impl RangeState {
    pub(super) fn capture(value: ByteRange) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct StartupState {
    ranges: Vec<RangeState>,
    playable_ms: u64,
    provenance: u8,
}

impl StartupState {
    pub(super) fn capture(value: &StartupFootprint) -> Self {
        Self {
            ranges: value
                .ranges()
                .iter()
                .copied()
                .map(RangeState::capture)
                .collect(),
            playable_ms: value.playable_ms(),
            provenance: match value.provenance() {
                StartupProvenance::WholeObjectV1 => 0,
                StartupProvenance::ClassicMp4V1 => 1,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum RequestState {
    Range {
        bytes: RangeState,
        promotion_bytes: Option<u64>,
        promotion_until_ms: Option<u64>,
    },
    Whole {
        maximum_bytes: u64,
        exact: bool,
        reason: u8,
    },
}

impl RequestState {
    pub(super) fn capture(value: RetrievalRequest) -> Self {
        match value {
            RetrievalRequest::FetchRange { bytes, promotion } => Self::Range {
                bytes: RangeState::capture(bytes),
                promotion_bytes: promotion.map(|item| item.maximum_bytes),
                promotion_until_ms: promotion.map(|item| item.valid_until_ms),
            },
            RetrievalRequest::FetchWhole { contract, reason } => Self::Whole {
                maximum_bytes: contract.maximum_bytes(),
                exact: matches!(contract, WholeBodyContract::Exact { .. }),
                reason: reason_code(reason),
            },
        }
    }
}

fn reason_code(value: WholeFetchReason) -> u8 {
    match value {
        WholeFetchReason::DirectCrossover => 0,
        WholeFetchReason::PromotedResponse => 1,
        WholeFetchReason::PlannedCompletion => 2,
    }
}
