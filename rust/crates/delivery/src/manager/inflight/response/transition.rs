use super::super::ResponsePhase;
use crate::chunk::downloader::{OpenedResponse, ResponseObservation, ResponseWriteMode};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::ByteRange;

pub(super) fn allowed(
    launched: RetrievalRequest,
    promotion_authorization: Option<ghostr_engine::adaptive::PromotionGrant>,
    phase: ResponsePhase,
    response: &OpenedResponse,
    opened_at_ms: u64,
) -> bool {
    if covered(launched, response) {
        return true;
    }
    match (launched, response.observation(), response.mode()) {
        (
            RetrievalRequest::FetchRange { bytes, .. },
            ResponseObservation::Partial { range, .. },
            ResponseWriteMode::Sparse,
        ) => contains(bytes, range),
        (
            RetrievalRequest::FetchRange { bytes, .. },
            ResponseObservation::Body {
                request:
                    RetrievalRequest::FetchRange {
                        bytes: returned, ..
                    },
                ..
            },
            ResponseWriteMode::Sparse,
        ) => contains(bytes, returned),
        (RetrievalRequest::FetchRange { .. }, observation, mode) => promoted(
            promotion_authorization,
            phase,
            observation,
            mode,
            opened_at_ms,
        ),
        (RetrievalRequest::FetchWhole { contract, reason }, observation, mode) => {
            whole(contract, reason, observation, mode)
        }
    }
}

pub(super) fn covered(launched: RetrievalRequest, response: &OpenedResponse) -> bool {
    let (
        RetrievalRequest::FetchRange { bytes, .. },
        ResponseObservation::Body {
            request:
                RetrievalRequest::FetchWhole {
                    contract: WholeBodyContract::Exact { expected_bytes },
                    reason: WholeFetchReason::PlannedCompletion,
                },
            promoted: false,
            ..
        },
        ResponseWriteMode::SingleResponse(WholeBodyContract::Exact {
            expected_bytes: mode_bytes,
        }),
    ) = (launched, response.observation(), response.mode())
    else {
        return false;
    };
    bytes.start == 0
        && expected_bytes > 0
        && expected_bytes <= bytes.end
        && mode_bytes == expected_bytes
}

fn promoted(
    grant: Option<ghostr_engine::adaptive::PromotionGrant>,
    phase: ResponsePhase,
    observation: ResponseObservation,
    mode: ResponseWriteMode,
    opened_at_ms: u64,
) -> bool {
    let Some(grant) = grant.filter(|grant| opened_at_ms <= grant.valid_until_ms) else {
        return false;
    };
    let ResponseObservation::Body {
        request: RetrievalRequest::FetchWhole { contract, .. },
        promoted: true,
        ..
    } = observation
    else {
        return false;
    };
    mode == ResponseWriteMode::SingleResponse(contract)
        && contract.maximum_bytes() == grant.maximum_bytes
        && phase
            .opportunity()
            .is_some_and(|opportunity| opportunity.contract() == contract)
}

fn whole(
    granted: WholeBodyContract,
    reason: ghostr_engine::adaptive::WholeFetchReason,
    observation: ResponseObservation,
    mode: ResponseWriteMode,
) -> bool {
    let ResponseObservation::Body {
        request:
            RetrievalRequest::FetchWhole {
                contract,
                reason: seen,
            },
        promoted: false,
        ..
    } = observation
    else {
        return false;
    };
    mode == ResponseWriteMode::SingleResponse(contract)
        && seen == reason
        && contract.maximum_bytes() <= granted.maximum_bytes()
}

fn contains(granted: ByteRange, returned: ByteRange) -> bool {
    granted.start <= returned.start && returned.end <= granted.end && !returned.is_empty()
}
