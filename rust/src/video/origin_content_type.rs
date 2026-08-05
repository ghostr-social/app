//! Admission policy for media bytes received from an origin.

use anyhow::Result;
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub(crate) struct UnsupportedOriginMediaType(String);

impl Display for UnsupportedOriginMediaType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "origin returned unsupported Content-Type {:?}",
            self.0
        )
    }
}

impl std::error::Error for UnsupportedOriginMediaType {}

pub(crate) fn require_admissible(headers: &HeaderMap) -> Result<()> {
    let Some(value) = headers.get(CONTENT_TYPE) else {
        return Ok(());
    };
    let value = value
        .to_str()
        .map_err(|_| UnsupportedOriginMediaType("<non-text>".to_owned()))?;
    let mime = value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match admissible(&mime) {
        true => Ok(()),
        false => Err(UnsupportedOriginMediaType(mime).into()),
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
