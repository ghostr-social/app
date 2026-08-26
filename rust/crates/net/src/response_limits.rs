use anyhow::{ensure, Result};
use reqwest::header::HeaderMap;

pub const MAX_MEDIA_RESPONSE_HEADER_BYTES: usize = 32 * 1024;
const MAX_MEDIA_RESPONSE_HEADERS: usize = 128;
const HEADER_WIRE_OVERHEAD: usize = 4;

/// # Errors
///
/// Returns an error when the response has too many headers or exceeds the header-byte limit.
pub fn validate_response_headers(headers: &HeaderMap) -> Result<()> {
    ensure!(
        headers.len() <= MAX_MEDIA_RESPONSE_HEADERS,
        "media response has too many headers"
    );
    let bytes = headers.iter().fold(0_usize, |total, (name, value)| {
        total
            .saturating_add(name.as_str().len())
            .saturating_add(value.as_bytes().len())
            .saturating_add(HEADER_WIRE_OVERHEAD)
    });
    ensure!(
        bytes <= MAX_MEDIA_RESPONSE_HEADER_BYTES,
        "media response headers exceed byte limit"
    );
    Ok(())
}
