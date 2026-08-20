use serde::{Deserialize, Serialize};

use crate::adaptive::{PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason};
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

    pub(super) fn range(self) -> ByteRange {
        ByteRange::new(self.start, self.end)
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

    pub(super) fn startup(&self) -> StartupFootprint {
        let provenance = match self.provenance {
            0 => StartupProvenance::WholeObjectV1,
            _ => StartupProvenance::ClassicMp4V1,
        };
        StartupFootprint::new(
            self.ranges.iter().copied().map(RangeState::range).collect(),
            self.playable_ms,
            provenance,
        )
        .expect("captured startup footprint remains valid")
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

    pub(super) fn request(self) -> RetrievalRequest {
        match self {
            Self::Range {
                bytes,
                promotion_bytes,
                promotion_until_ms,
            } => range_request(bytes, promotion_bytes, promotion_until_ms),
            Self::Whole {
                maximum_bytes,
                exact,
                reason,
            } => whole_request(maximum_bytes, exact, reason),
        }
    }
}

fn range_request(bytes: RangeState, maximum: Option<u64>, until: Option<u64>) -> RetrievalRequest {
    let promotion = maximum
        .zip(until)
        .map(|(maximum_bytes, valid_until_ms)| PromotionGrant {
            maximum_bytes,
            valid_until_ms,
        });
    RetrievalRequest::FetchRange {
        bytes: bytes.range(),
        promotion,
    }
}

fn whole_request(maximum: u64, exact: bool, reason: u8) -> RetrievalRequest {
    let contract = match exact {
        true => WholeBodyContract::Exact {
            expected_bytes: maximum,
        },
        false => WholeBodyContract::Capped {
            maximum_bytes: maximum,
        },
    };
    RetrievalRequest::FetchWhole {
        contract,
        reason: fetch_reason(reason),
    }
}

fn reason_code(value: WholeFetchReason) -> u8 {
    match value {
        WholeFetchReason::DirectCrossover => 0,
        WholeFetchReason::PromotedResponse => 1,
        WholeFetchReason::PlannedCompletion => 2,
    }
}

fn fetch_reason(value: u8) -> WholeFetchReason {
    match value {
        0 => WholeFetchReason::DirectCrossover,
        1 => WholeFetchReason::PromotedResponse,
        _ => WholeFetchReason::PlannedCompletion,
    }
}
