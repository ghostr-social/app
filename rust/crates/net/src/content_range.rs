//! Parser for HTTP `Content-Range` headers of the form
//! `bytes <start>-<end>/<total>`.

use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedContentRange {
    pub range: Range<u64>,
    pub total: Option<u64>,
}

/// Returns the exact half-open response span and optional complete
/// length. `None` means the header is malformed or internally invalid.
pub fn parse(value: &str) -> Option<ParsedContentRange> {
    let spec = value.trim().strip_prefix("bytes")?.trim_start();
    let (range_part, total_part) = spec.split_once('/')?;
    let (start, end) = range_part.split_once('-')?;
    let start = start.trim().parse().ok()?;
    let end = end.trim().parse::<u64>().ok()?.checked_add(1)?;
    if end <= start {
        return None;
    }
    let total = match total_part.trim() {
        "*" => None,
        value => Some(value.parse().ok()?),
    };
    if total.is_some_and(|total| end > total) {
        return None;
    }
    Some(ParsedContentRange {
        range: start..end,
        total,
    })
}
