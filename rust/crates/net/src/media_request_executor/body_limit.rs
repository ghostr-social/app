use anyhow::{ensure, Context as _, Result};
use reqwest::{Method, Request};

const DEFAULT_BODY_LIMIT: u64 = 8 * 1024 * 1024;

pub(super) fn maximum(request: &Request, explicit: Option<u64>) -> Result<u64> {
    if request.method() == Method::HEAD {
        return Ok(0);
    }
    if let Some(maximum) = explicit {
        return Ok(maximum);
    }
    let Some(range) = request.headers().get(reqwest::header::RANGE) else {
        return Ok(DEFAULT_BODY_LIMIT);
    };
    let range = range
        .to_str()?
        .strip_prefix("bytes=")
        .context("invalid media range")?;
    let (start, end) = range.split_once('-').context("invalid media range")?;
    if start.is_empty() || end.is_empty() {
        return Ok(DEFAULT_BODY_LIMIT);
    }
    let start: u64 = start.parse()?;
    let end: u64 = end.parse()?;
    ensure!(end >= start, "reversed media range");
    end.checked_sub(start)
        .and_then(|length| length.checked_add(1))
        .context("media range overflow")
}
