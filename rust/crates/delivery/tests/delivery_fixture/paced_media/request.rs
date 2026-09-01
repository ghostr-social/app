use axum::http::{header, HeaderMap};

pub(super) fn parse(headers: &HeaderMap, total: u64) -> Option<(u64, u64)> {
    let value = headers.get(header::RANGE)?;
    let value = value.to_str().expect("valid test fixture");
    let value = value.strip_prefix("bytes=").expect("valid test fixture");
    let (start, end) = value.split_once('-').expect("valid test fixture");
    Some((
        start.parse().expect("valid test fixture"),
        end.parse().unwrap_or(total - 1).min(total - 1),
    ))
}
