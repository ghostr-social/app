//! Validation and range classification for chunk origin responses.

use anyhow::{bail, ensure, Context, Result};
use ghostr_engine::ByteRange;
use ghostr_net::{content_range, origin_content_type};
use reqwest::header::CONTENT_RANGE;
use reqwest::{Response, StatusCode};

pub(crate) enum RangeReply {
    Partial { total: Option<u64> },
    FullBody,
    Ignored,
}

pub(crate) fn classify(response: &Response, range: ByteRange) -> Result<RangeReply> {
    origin_content_type::require_admissible(response.headers())?;
    if response.status() == StatusCode::PARTIAL_CONTENT {
        let total = verified_total(response, range.start)?;
        return Ok(RangeReply::Partial { total });
    }
    if range.start == 0 {
        return Ok(RangeReply::FullBody);
    }
    Ok(RangeReply::Ignored)
}

fn verified_total(response: &Response, expected_start: u64) -> Result<Option<u64>> {
    let header = response.headers().get(CONTENT_RANGE);
    let Some(value) = header.and_then(|value| value.to_str().ok()) else {
        bail!("partial content response is missing Content-Range");
    };
    let (start, total) = content_range::parse(value).context("unparseable Content-Range")?;
    ensure!(
        start == expected_start,
        "server answered a different range offset"
    );
    Ok(total)
}
