//! Admission policy for media bytes received from an origin.

use anyhow::Result;
use core::fmt::{Display, Formatter};
use reqwest::header::{HeaderMap, CONTENT_TYPE};

#[derive(Debug)]
pub struct UnsupportedOriginMediaType(String);

impl Display for UnsupportedOriginMediaType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "origin returned unsupported Content-Type {:?}",
            self.0
        )
    }
}

impl core::error::Error for UnsupportedOriginMediaType {}

/// # Errors
///
/// Returns an error when the content type is malformed or outside the media allowlist.
pub fn require_admissible(headers: &HeaderMap) -> Result<()> {
    let Some(value) = headers.get(CONTENT_TYPE) else {
        return Ok(());
    };
    let value = value
        .to_str()
        .map_err(|_invalid_header| UnsupportedOriginMediaType("<non-text>".to_owned()))?;
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if admissible(&mime) {
        Ok(())
    } else {
        Err(UnsupportedOriginMediaType(mime).into())
    }
}

fn admissible(mime: &str) -> bool {
    mime.starts_with("video/")
        || matches!(
            mime,
            "application/octet-stream"
                | "binary/octet-stream"
                | "application/vnd.apple.mpegurl"
                | "application/x-mpegurl"
                | "audio/mpegurl"
                | "audio/x-mpegurl"
        )
}
