use axum::http::header::RANGE;
use axum::http::{HeaderMap, HeaderValue};
use ghostr_net::content_range::{parse_range_decimal, ParsedContentRange};
use ghostr_net::media_request_executor::MediaRequest;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AssetRangeRequest {
    Full,
    Bounded { start: u64, last: u64 },
    Open { start: u64 },
    Suffix { length: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResolvedAssetRange {
    Full,
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

impl AssetRangeRequest {
    pub(super) fn collect(headers: &HeaderMap) -> Self {
        let mut values = headers.get_all(RANGE).iter();
        let Some(value) = values.next() else {
            return Self::Full;
        };
        if values.next().is_some() {
            return Self::Full;
        }
        parsed(value).unwrap_or(Self::Full)
    }

    pub(super) fn apply(self, request: MediaRequest) -> anyhow::Result<MediaRequest> {
        match self.header_value() {
            Some(value) => Ok(request.header(RANGE, value.parse()?)),
            None => Ok(request),
        }
    }

    pub(super) fn resolve(self, total: u64) -> ResolvedAssetRange {
        let bounds = match self {
            Self::Full => return ResolvedAssetRange::Full,
            Self::Bounded { start, last } => (start, last.saturating_add(1).min(total)),
            Self::Open { start } => (start, total),
            Self::Suffix { length } => (total.saturating_sub(length), total),
        };
        if bounds.0 >= bounds.1 {
            ResolvedAssetRange::Unsatisfiable
        } else {
            ResolvedAssetRange::Partial {
                start: bounds.0,
                end: bounds.1,
            }
        }
    }

    pub(super) fn accepts(self, response: &ParsedContentRange) -> bool {
        match self {
            Self::Bounded { start, last } => bounded_response(response, start, last),
            Self::Open { start } => response.range.start == start,
            Self::Suffix { .. } => suffix_response(self, response),
            Self::Full => false,
        }
    }

    pub(super) fn is_ranged(self) -> bool {
        self != Self::Full
    }

    pub(super) fn locally_unsatisfiable(self) -> bool {
        matches!(self, Self::Suffix { length: 0 })
    }

    pub(super) fn is_unsatisfiable(self, total: u64) -> bool {
        self.resolve(total) == ResolvedAssetRange::Unsatisfiable
    }

    fn header_value(self) -> Option<String> {
        match self {
            Self::Full => None,
            Self::Bounded { start, last } => Some(format!("bytes={start}-{last}")),
            Self::Open { start } => Some(format!("bytes={start}-")),
            Self::Suffix { length } => Some(format!("bytes=-{length}")),
        }
    }
}

fn bounded_response(response: &ParsedContentRange, start: u64, last: u64) -> bool {
    response.range.start == start && response.range.end <= last.saturating_add(1)
}

fn suffix_response(request: AssetRangeRequest, response: &ParsedContentRange) -> bool {
    let Some(total) = response.total else {
        return false;
    };
    let ResolvedAssetRange::Partial { start, end } = request.resolve(total) else {
        return false;
    };
    response.range.start == start && response.range.end <= end
}

fn parsed(value: &HeaderValue) -> Option<AssetRangeRequest> {
    let (unit, spec) = value.to_str().ok()?.split_once('=')?;
    if !unit.trim().eq_ignore_ascii_case("bytes") || spec.contains(',') {
        return None;
    }
    let (from, to) = spec.trim().split_once('-')?;
    parsed_bounds(from.trim(), to.trim())
}

fn parsed_bounds(from: &str, to: &str) -> Option<AssetRangeRequest> {
    match (from.is_empty(), to.is_empty()) {
        (true, true) => None,
        (true, false) => Some(AssetRangeRequest::Suffix {
            length: parse_range_decimal(to)?,
        }),
        (false, true) => Some(AssetRangeRequest::Open {
            start: parse_range_decimal(from)?,
        }),
        (false, false) => {
            let start = parse_range_decimal(from)?;
            let last = parse_range_decimal(to)?;
            (start <= last).then_some(AssetRangeRequest::Bounded { start, last })
        }
    }
}
