//! Validation and semantic classification for origin responses.

use anyhow::{ensure, Context, Result};
use ghostr_engine::adaptive::{
    PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaResponse;
use ghostr_net::{content_range, origin_content_type};
use reqwest::header::CONTENT_RANGE;
use reqwest::StatusCode;

pub(crate) enum ResponseReply {
    Partial {
        range: ByteRange,
        total: Option<u64>,
    },
    Body {
        request: RetrievalRequest,
        range_support: Option<bool>,
        promoted: bool,
    },
    Ignored {
        range_support: Option<bool>,
    },
}

pub(crate) fn classify(
    response: &MediaResponse,
    request: RetrievalRequest,
    conditional: bool,
) -> Result<ResponseReply> {
    origin_content_type::require_admissible(response.headers())?;
    match request {
        RetrievalRequest::FetchRange { bytes, promotion } => {
            classify_range(response, bytes, promotion, conditional)
        }
        RetrievalRequest::FetchWhole { contract, reason } => {
            classify_whole(response, contract, reason)
        }
    }
}

fn classify_range(
    response: &MediaResponse,
    expected: ByteRange,
    promotion: Option<PromotionGrant>,
    conditional: bool,
) -> Result<ResponseReply> {
    if response.status() == StatusCode::PARTIAL_CONTENT {
        let (range, total) = verified_range(response, expected)?;
        return Ok(ResponseReply::Partial { range, total });
    }
    ensure!(
        response.status() == StatusCode::OK,
        "range response is not 200 or 206"
    );
    let range_support = (!conditional).then_some(false);
    if let Some(grant) = promotion {
        validate_maximum(response.content_length(), grant.maximum_bytes)?;
        return Ok(ResponseReply::Body {
            request: RetrievalRequest::FetchWhole {
                contract: whole_contract(response.content_length(), grant.maximum_bytes),
                reason: WholeFetchReason::PromotedResponse,
            },
            range_support,
            promoted: true,
        });
    }
    bounded_or_ignored(response, expected, range_support)
}

fn classify_whole(
    response: &MediaResponse,
    contract: WholeBodyContract,
    reason: WholeFetchReason,
) -> Result<ResponseReply> {
    ensure!(
        response.status() == StatusCode::OK,
        "whole response is not 200"
    );
    validate_contract(response.content_length(), contract)?;
    let contract = effective_contract(response.content_length(), contract);
    Ok(ResponseReply::Body {
        request: RetrievalRequest::FetchWhole { contract, reason },
        range_support: None,
        promoted: false,
    })
}

fn effective_contract(length: Option<u64>, contract: WholeBodyContract) -> WholeBodyContract {
    match (length, contract) {
        (Some(expected_bytes), WholeBodyContract::Capped { .. }) => {
            WholeBodyContract::Exact { expected_bytes }
        }
        _ => contract,
    }
}

fn bounded_or_ignored(
    response: &MediaResponse,
    expected: ByteRange,
    range_support: Option<bool>,
) -> Result<ResponseReply> {
    if expected.start != 0 {
        return Ok(ResponseReply::Ignored { range_support });
    }
    let request = match response.content_length() {
        Some(length) if length > 0 && length <= expected.end => RetrievalRequest::FetchWhole {
            contract: WholeBodyContract::Exact {
                expected_bytes: length,
            },
            reason: WholeFetchReason::PlannedCompletion,
        },
        _ => RetrievalRequest::FetchRange {
            bytes: expected,
            promotion: None,
        },
    };
    Ok(ResponseReply::Body {
        request,
        range_support,
        promoted: false,
    })
}

fn validate_contract(length: Option<u64>, contract: WholeBodyContract) -> Result<()> {
    match contract {
        WholeBodyContract::Exact { expected_bytes } => {
            ensure!(expected_bytes > 0, "whole response length must be positive");
            if let Some(length) = length {
                ensure!(length == expected_bytes, "whole response length changed");
            }
        }
        WholeBodyContract::Capped { maximum_bytes } => {
            validate_maximum(length, maximum_bytes)?;
        }
    }
    Ok(())
}

fn validate_maximum(length: Option<u64>, maximum: u64) -> Result<()> {
    ensure!(maximum > 0, "whole response cap must be positive");
    if let Some(length) = length {
        ensure!(length > 0, "whole response length must be positive");
        ensure!(length <= maximum, "whole response exceeds its hard cap");
    }
    Ok(())
}

fn whole_contract(length: Option<u64>, maximum: u64) -> WholeBodyContract {
    match length {
        Some(expected_bytes) => WholeBodyContract::Exact { expected_bytes },
        None => WholeBodyContract::Capped {
            maximum_bytes: maximum,
        },
    }
}

fn verified_range(
    response: &MediaResponse,
    expected: ByteRange,
) -> Result<(ByteRange, Option<u64>)> {
    let parsed = verified_content_range(response)?;
    let returned = ByteRange::new(parsed.range.start, parsed.range.end);
    ensure!(
        returned.start == expected.start,
        "server answered a different range offset"
    );
    ensure!(
        returned.end <= expected.end,
        "server answered beyond the requested range"
    );
    if let Some(length) = response.content_length() {
        ensure!(
            length == returned.len(),
            "partial response Content-Range length differs from Content-Length"
        );
    }
    Ok((returned, parsed.total))
}

fn verified_content_range(response: &MediaResponse) -> Result<content_range::ParsedContentRange> {
    let mut values = response.headers().get_all(CONTENT_RANGE).iter();
    let value = values
        .next()
        .context("partial content response is missing Content-Range")?;
    ensure!(values.next().is_none(), "duplicate Content-Range");
    let text = value.to_str().context("invalid Content-Range text")?;
    content_range::parse(text).context("unparseable Content-Range")
}
