use super::{invalid, invalid_error, valid};
use anyhow::Context;
use ghostr_net::strong_etag::{single_strong_etag, StrongEtag};
use reqwest::header::{HeaderMap, HeaderName, CONTENT_LENGTH};

pub(super) fn required_length(headers: &HeaderMap) -> Result<u64, super::super::FetchProblem> {
    let value = single_header(headers, CONTENT_LENGTH)?
        .context("missing HLS Content-Length")
        .map_err(invalid_error)?;
    ghostr_net::content_range::parse_range_decimal(value)
        .context("invalid HLS Content-Length")
        .map_err(invalid_error)
}

pub(super) fn strong_etag(
    headers: &HeaderMap,
) -> Result<Option<StrongEtag>, super::super::FetchProblem> {
    single_strong_etag(headers).map_err(|_| invalid("invalid or duplicate HLS ETag"))
}

pub(super) fn single_header(
    headers: &HeaderMap,
    name: HeaderName,
) -> Result<Option<&str>, super::super::FetchProblem> {
    let mut values = headers.get_all(name).iter();
    let value = values.next();
    valid(values.next().is_none(), "duplicate HLS response header")?;
    value
        .map(|value| {
            value
                .to_str()
                .map_err(|_| invalid("invalid HLS response header text"))
        })
        .transpose()
}
