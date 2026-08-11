//! Validation and range classification for chunk origin responses.

use anyhow::{bail, ensure, Context, Result};
use ghostr_engine::ByteRange;
use ghostr_net::{content_range, origin_content_type};
use reqwest::header::CONTENT_RANGE;
use reqwest::{Response, StatusCode};

pub(crate) enum RangeReply {
    Partial {
        range: ByteRange,
        total: Option<u64>,
    },
    FullBody,
    Ignored,
}

pub(crate) fn classify(response: &Response, range: ByteRange) -> Result<RangeReply> {
    origin_content_type::require_admissible(response.headers())?;
    if response.status() == StatusCode::PARTIAL_CONTENT {
        let (returned, total) = verified_range(response, range)?;
        return Ok(RangeReply::Partial {
            range: returned,
            total,
        });
    }
    if range.start == 0 {
        return Ok(RangeReply::FullBody);
    }
    Ok(RangeReply::Ignored)
}

fn verified_range(response: &Response, expected: ByteRange) -> Result<(ByteRange, Option<u64>)> {
    let header = response.headers().get(CONTENT_RANGE);
    let Some(value) = header.and_then(|value| value.to_str().ok()) else {
        bail!("partial content response is missing Content-Range");
    };
    let parsed = content_range::parse(value).context("unparseable Content-Range")?;
    ensure!(
        parsed.range.start == expected.start,
        "server answered a different range offset"
    );
    ensure!(
        parsed.range.end <= expected.end,
        "server answered beyond the requested range"
    );
    Ok((
        ByteRange::new(parsed.range.start, parsed.range.end),
        parsed.total,
    ))
}
