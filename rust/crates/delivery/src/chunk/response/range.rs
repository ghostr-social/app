use anyhow::{ensure, Context as _, Result};
use ghostr_engine::ByteRange;
use ghostr_net::content_range;
use ghostr_net::media_request_executor::MediaResponse;
use reqwest::header::CONTENT_RANGE;

pub(super) fn verified(
    response: &MediaResponse,
    expected: ByteRange,
) -> Result<(ByteRange, Option<u64>)> {
    let parsed = content_range(response)?;
    let returned = ByteRange::new(parsed.range.start, parsed.range.end);
    ensure!(
        returned.start == expected.start,
        "server answered a different range offset"
    );
    ensure!(
        returned.end <= expected.end,
        "server answered beyond the requested range"
    );
    if let Some(length) = response.content_length() {
        ensure!(
            length == returned.len(),
            "partial response Content-Range length differs from Content-Length"
        );
    }
    Ok((returned, parsed.total))
}

fn content_range(response: &MediaResponse) -> Result<content_range::ParsedContentRange> {
    let mut values = response.headers().get_all(CONTENT_RANGE).iter();
    let value = values
        .next()
        .context("partial content response is missing Content-Range")?;
    ensure!(values.next().is_none(), "duplicate Content-Range");
    let text = value.to_str().context("invalid Content-Range text")?;
    content_range::parse(text).context("unparseable Content-Range")
}
