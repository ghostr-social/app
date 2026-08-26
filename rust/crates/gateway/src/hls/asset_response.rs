use crate::hls::asset_request::AssetRangeRequest;
use anyhow::{bail, ensure, Context as _, Result};
use axum::body::Body;
use axum::http::header::{CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{HeaderMap, Response, StatusCode};
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use ghostr_net::content_range;
use ghostr_net::media_request_executor::MediaResponse;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AssetBodyContract {
    Empty,
    Exact { bytes: u64 },
    Capped { maximum: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AssetResponseEnvelope {
    Full {
        length: Option<u64>,
    },
    Partial {
        start: u64,
        end: u64,
        total: Option<u64>,
    },
    Unsatisfiable {
        total: Option<u64>,
    },
}

pub(super) fn validate(
    request: AssetRangeRequest,
    response: &MediaResponse,
) -> Result<AssetResponseEnvelope> {
    match (request.is_ranged(), response.status()) {
        (false, StatusCode::OK) => full(response),
        (true, StatusCode::PARTIAL_CONTENT) => partial(request, response),
        (true, StatusCode::RANGE_NOT_SATISFIABLE) => unsatisfiable(request, response.headers()),
        _ => bail!("HLS asset response does not match its request"),
    }
}

pub(super) fn local_unsatisfiable() -> Result<Response<Body>, StatusCode> {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_LENGTH, 0)
        .body(Body::empty())
        .map_err(|error| {
            log::warn!("Could not build local HLS range response: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

impl AssetResponseEnvelope {
    pub(super) fn status(self) -> StatusCode {
        match self {
            Self::Full { .. } => StatusCode::OK,
            Self::Partial { .. } => StatusCode::PARTIAL_CONTENT,
            Self::Unsatisfiable { .. } => StatusCode::RANGE_NOT_SATISFIABLE,
        }
    }

    pub(super) fn content_length(self) -> Option<u64> {
        match self {
            Self::Full { length } => length,
            Self::Partial { start, end, .. } => Some(end - start),
            Self::Unsatisfiable { .. } => Some(0),
        }
    }

    pub(super) fn content_range(self) -> Option<String> {
        match self {
            Self::Partial {
                start, end, total, ..
            } => Some(format!(
                "bytes {start}-{}/{}",
                end - 1,
                total.map_or_else(|| "*".to_owned(), |value| value.to_string())
            )),
            Self::Unsatisfiable { total: Some(total) } => Some(format!("bytes */{total}")),
            _ => None,
        }
    }

    pub(super) fn advertises_ranges(self) -> bool {
        matches!(self, Self::Partial { .. })
    }

    pub(super) fn body_contract(self) -> AssetBodyContract {
        match self {
            Self::Full {
                length: Some(bytes),
            } => AssetBodyContract::Exact { bytes },
            Self::Partial { start, end, .. } => AssetBodyContract::Exact { bytes: end - start },
            Self::Full { length: None } => AssetBodyContract::Capped {
                maximum: MAX_HLS_ASSET_BYTES as u64,
            },
            Self::Unsatisfiable { .. } => AssetBodyContract::Empty,
        }
    }
}

impl AssetBodyContract {
    pub(super) fn checked_total(self, sent: u64, next: usize) -> Option<u64> {
        let total = sent.checked_add(next as u64)?;
        (total <= self.maximum()).then_some(total)
    }

    pub(super) fn complete(self, sent: u64) -> bool {
        match self {
            Self::Empty => sent == 0,
            Self::Exact { bytes } => sent == bytes,
            Self::Capped { .. } => true,
        }
    }

    fn maximum(self) -> u64 {
        match self {
            Self::Empty => 0,
            Self::Exact { bytes } => bytes,
            Self::Capped { maximum } => maximum,
        }
    }
}

fn full(response: &MediaResponse) -> Result<AssetResponseEnvelope> {
    ensure!(
        response
            .headers()
            .get_all(CONTENT_RANGE)
            .iter()
            .next()
            .is_none(),
        "full HLS asset response carries Content-Range"
    );
    let length = response.content_length();
    ensure!(
        length.is_none_or(|bytes| bytes <= MAX_HLS_ASSET_BYTES as u64),
        "HLS asset exceeds its byte limit"
    );
    Ok(AssetResponseEnvelope::Full { length })
}

fn partial(request: AssetRangeRequest, response: &MediaResponse) -> Result<AssetResponseEnvelope> {
    ensure!(
        !is_multipart(response.headers()),
        "multipart HLS range response"
    );
    let value = single_content_range(response.headers())?;
    let parsed = content_range::parse(value).context("invalid HLS asset Content-Range")?;
    ensure!(request.accepts(&parsed), "HLS asset response range changed");
    let length = parsed.range.end - parsed.range.start;
    ensure!(
        length <= MAX_HLS_ASSET_BYTES as u64,
        "HLS asset range exceeds its byte limit"
    );
    if let Some(declared) = response.content_length() {
        ensure!(declared == length, "HLS asset response length changed");
    }
    Ok(AssetResponseEnvelope::Partial {
        start: parsed.range.start,
        end: parsed.range.end,
        total: parsed.total,
    })
}

fn unsatisfiable(request: AssetRangeRequest, headers: &HeaderMap) -> Result<AssetResponseEnvelope> {
    let total = content_range::parse_unsatisfied(single_content_range(headers)?)
        .context("invalid unsatisfied HLS Content-Range")?;
    ensure!(request.is_unsatisfiable(total), "HLS range is satisfiable");
    Ok(AssetResponseEnvelope::Unsatisfiable { total: Some(total) })
}

fn single_content_range(headers: &HeaderMap) -> Result<&str> {
    let mut values = headers.get_all(CONTENT_RANGE).iter();
    let value = values.next().context("missing HLS Content-Range")?;
    ensure!(values.next().is_none(), "duplicate HLS Content-Range");
    value.to_str().context("invalid HLS Content-Range text")
}

fn is_multipart(headers: &HeaderMap) -> bool {
    headers.get_all(CONTENT_TYPE).iter().any(is_multipart_value)
}

fn is_multipart_value(value: &axum::http::HeaderValue) -> bool {
    value
        .to_str()
        .ok()
        .and_then(|text| text.split(';').next())
        .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("multipart/byteranges"))
}
