use axum::http::{header, HeaderMap};
use core::ops::Range;

pub(super) fn requested_range(headers: &HeaderMap, total: u64) -> Option<Range<u64>> {
    let value = headers.get(header::RANGE)?.to_str().expect("range header");
    let (start, end) = value
        .trim_start_matches("bytes=")
        .split_once('-')
        .expect("valid test fixture");
    let start = start.parse().expect("valid test fixture");
    let end = end.parse::<u64>().unwrap_or(total - 1).min(total - 1);
    Some(start..end + 1)
}
