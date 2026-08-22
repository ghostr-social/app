use axum::http::header::RANGE;
use axum::http::{HeaderMap, HeaderValue};
use ghostr_net::content_range::parse_range_decimal;

/// A `Range` request header resolved against the total video length.
/// `Partial` bounds are half-open `[start, end)` with `end <= total`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolvedRange {
    Full,
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

/// Resolves one optional `Range` field value against a known total length.
pub fn resolve(header: Option<&HeaderValue>, total: u64) -> ResolvedRange {
    match header.and_then(parsed_spec) {
        Some(spec) => resolved_spec(spec, total),
        None => ResolvedRange::Full,
    }
}

/// Resolves exactly one `Range` field; absent or duplicate fields are ignored.
pub(crate) fn resolve_all(headers: &HeaderMap, total: u64) -> ResolvedRange {
    let mut values = headers.get_all(RANGE).iter();
    let Some(value) = values.next() else {
        return ResolvedRange::Full;
    };
    if values.next().is_some() {
        return ResolvedRange::Full;
    }
    resolve(Some(value), total)
}

enum Spec {
    FromTo(u64, u64),
    From(u64),
    Suffix(u64),
}

fn parsed_spec(value: &HeaderValue) -> Option<Spec> {
    let text = value.to_str().ok()?;
    let (unit, spec) = text.split_once('=')?;
    if !unit.trim().eq_ignore_ascii_case("bytes") {
        return None;
    }
    let spec = spec.trim();
    if spec.contains(',') {
        return None;
    }
    let (from, to) = spec.split_once('-')?;
    parsed_bounds(from.trim(), to.trim())
}

fn parsed_bounds(from: &str, to: &str) -> Option<Spec> {
    match (from.is_empty(), to.is_empty()) {
        (true, true) => None,
        (true, false) => Some(Spec::Suffix(parse_range_decimal(to)?)),
        (false, true) => Some(Spec::From(parse_range_decimal(from)?)),
        (false, false) => Some(Spec::FromTo(
            parse_range_decimal(from)?,
            parse_range_decimal(to)?,
        )),
    }
}

fn resolved_spec(spec: Spec, total: u64) -> ResolvedRange {
    let Some((start, end)) = resolved_bounds(spec, total) else {
        return ResolvedRange::Full;
    };
    if start >= end {
        return ResolvedRange::Unsatisfiable;
    }
    ResolvedRange::Partial { start, end }
}

fn resolved_bounds(spec: Spec, total: u64) -> Option<(u64, u64)> {
    match spec {
        Spec::FromTo(start, last) => from_to_bounds(start, last, total),
        Spec::From(start) => Some((start, total)),
        Spec::Suffix(len) => Some((total.saturating_sub(len), total)),
    }
}

fn from_to_bounds(start: u64, last: u64, total: u64) -> Option<(u64, u64)> {
    if start > last {
        return None;
    }
    Some((start, last.saturating_add(1).min(total)))
}
