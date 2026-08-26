//! Parser for HTTP `Content-Range` headers of the form
//! `bytes <start>-<end>/<total>`.

use core::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedContentRange {
    pub range: Range<u64>,
    pub total: Option<u64>,
}

/// Returns the exact half-open response span and optional complete
/// length. `None` means the header is malformed or internally invalid.
pub fn parse(value: &str) -> Option<ParsedContentRange> {
    let spec = range_spec(value)?;
    let (range_part, total_part) = spec.split_once('/')?;
    let (start, end) = range_part.split_once('-')?;
    let start = parse_range_decimal(start.trim())?;
    let end = parse_range_decimal(end.trim())?.checked_add(1)?;
    if end <= start {
        return None;
    }
    let total = match total_part.trim() {
        "*" => None,
        value => Some(parse_range_decimal(value)?),
    };
    if total.is_some_and(|total| end > total) {
        return None;
    }
    Some(ParsedContentRange {
        range: start..end,
        total,
    })
}

/// Returns the complete representation length from `bytes */<length>`.
pub fn parse_unsatisfied(value: &str) -> Option<u64> {
    parse_range_decimal(range_spec(value)?.strip_prefix("*/")?.trim())
}

/// Parses one non-empty RFC range decimal without accepting a sign.
pub fn parse_range_decimal(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn range_spec(value: &str) -> Option<&str> {
    let value = value.trim();
    let boundary = value.find(|character: char| character.is_ascii_whitespace())?;
    let (unit, spec) = value.split_at(boundary);
    unit.eq_ignore_ascii_case("bytes")
        .then_some(spec.trim_start())
}
