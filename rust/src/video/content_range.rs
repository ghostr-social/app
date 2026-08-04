//! Parser for HTTP `Content-Range` headers of the form
//! `bytes <start>-<end>/<total>`.

/// Returns `(start, total)`; `total` is `None` for an unknown (`*`)
/// complete length. `None` when the header is not a byte-range spec.
pub fn parse(value: &str) -> Option<(u64, Option<u64>)> {
    let spec = value.trim().strip_prefix("bytes")?.trim_start();
    let (range_part, total_part) = spec.split_once('/')?;
    let (start, _end) = range_part.split_once('-')?;
    let total = total_part.trim().parse().ok();
    Some((start.trim().parse().ok()?, total))
}
