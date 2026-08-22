use super::{FetchSpec, ObjectContinuation};
use anyhow::Context;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_net::content_range;
use ghostr_net::identity_encoding::require_identity_encoding;
use ghostr_net::media_request_executor::MediaResponse;
use ghostr_net::response_limits::validate_response_headers;
use ghostr_net::strong_etag::StrongEtag;
use reqwest::header::CONTENT_RANGE;
use reqwest::StatusCode;
use url::Url;

mod generation;
mod headers;
use headers::{required_length, single_header, strong_etag};

pub(super) struct ResponseExtent {
    start: u64,
    end: u64,
    total: u64,
    strong_etag: Option<StrongEtag>,
}

impl ResponseExtent {
    pub(super) fn expected_bytes(&self) -> usize {
        usize::try_from(self.end - self.start).expect("bounded HLS response length")
    }

    pub(super) fn offset(&self) -> u64 {
        self.start
    }

    pub(super) fn continuation(&self, final_url: &Url) -> Option<ObjectContinuation> {
        (self.end < self.total).then(|| ObjectContinuation {
            next_offset: self.end,
            total: self.total,
            final_url: final_url.clone(),
            strong_etag: self
                .strong_etag
                .clone()
                .expect("incomplete HLS range has a strong ETag"),
        })
    }
}

pub(super) fn validate(
    response: &MediaResponse,
    spec: FetchSpec<'_>,
) -> Result<ResponseExtent, super::FetchProblem> {
    validate_response_headers(response.headers()).map_err(|error| invalid(error.to_string()))?;
    generation::validate_status(response.status(), spec.object.offset)?;
    validate_common(response)?;
    match response.status() {
        StatusCode::OK => full(response, spec),
        StatusCode::PARTIAL_CONTENT => partial(response, spec),
        status => Err(invalid(format!("ranged HLS response has status {status}"))),
    }
}

fn validate_common(response: &MediaResponse) -> Result<(), super::FetchProblem> {
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let error = anyhow::anyhow!("HLS object request failed with HTTP status {status}");
        return Err(super::FetchProblem::http(error, status));
    }
    require_identity_encoding(response.headers())
        .context("encoded HLS object is not cacheable")
        .map_err(|error| invalid(error.to_string()))
}

fn full(
    response: &MediaResponse,
    spec: FetchSpec<'_>,
) -> Result<ResponseExtent, super::FetchProblem> {
    generation(
        spec.object.offset == 0,
        "continued HLS range returned a full response",
    )?;
    valid(
        single_header(response.headers(), CONTENT_RANGE)?.is_none(),
        "full HLS response carries Content-Range",
    )?;
    let length = required_length(response.headers())?;
    valid(length > 0, "empty HLS object response")?;
    range(
        length <= spec.limit as u64 && length <= spec.object_limit,
        "full HLS object exceeds its byte grant",
    )?;
    Ok(ResponseExtent {
        start: 0,
        end: length,
        total: length,
        strong_etag: None,
    })
}

fn partial(
    response: &MediaResponse,
    spec: FetchSpec<'_>,
) -> Result<ResponseExtent, super::FetchProblem> {
    let value = single_header(response.headers(), CONTENT_RANGE)?
        .context("missing HLS Content-Range")
        .map_err(invalid_error)?;
    let parsed = content_range::parse(value)
        .context("invalid HLS Content-Range")
        .map_err(invalid_error)?;
    let total = parsed
        .total
        .context("unknown HLS object length")
        .map_err(invalid_error)?;
    validate_geometry(spec, parsed.range.start, parsed.range.end, total)?;
    let length = required_length(response.headers())?;
    range(
        length == parsed.range.end - parsed.range.start,
        "HLS range length changed",
    )?;
    let strong_etag = strong_etag(response.headers())?;
    generation::validate(response.url(), spec, total, strong_etag.as_ref())?;
    range(
        parsed.range.end == total || strong_etag.is_some(),
        "incomplete HLS range lacks a strong ETag",
    )?;
    Ok(ResponseExtent {
        start: parsed.range.start,
        end: parsed.range.end,
        total,
        strong_etag,
    })
}

fn validate_geometry(
    spec: FetchSpec<'_>,
    start: u64,
    end: u64,
    total: u64,
) -> Result<(), super::FetchProblem> {
    let requested_end = spec.request_end();
    range(start == spec.object.offset, "HLS range start changed")?;
    range(
        total > 0 && total <= spec.object_limit,
        "HLS object exceeds its stage limit",
    )?;
    range(
        end == requested_end.min(total),
        "HLS range did not fill its byte grant",
    )
}

fn invalid(message: impl Into<String>) -> super::FetchProblem {
    super::FetchProblem::new(
        anyhow::anyhow!(message.into()),
        ErrorReason::InvalidResponse,
    )
}

fn invalid_error(error: anyhow::Error) -> super::FetchProblem {
    invalid(error.to_string())
}

fn valid(condition: bool, message: &'static str) -> Result<(), super::FetchProblem> {
    condition.then_some(()).ok_or_else(|| invalid(message))
}

fn range(condition: bool, message: &'static str) -> Result<(), super::FetchProblem> {
    condition.then_some(()).ok_or_else(|| {
        super::FetchProblem::new(anyhow::anyhow!(message), ErrorReason::RangeNoncompliant)
    })
}

fn generation(condition: bool, message: &'static str) -> Result<(), super::FetchProblem> {
    condition.then_some(()).ok_or_else(|| {
        super::FetchProblem::restart_object(
            anyhow::anyhow!(message),
            ErrorReason::RangeNoncompliant,
        )
    })
}
