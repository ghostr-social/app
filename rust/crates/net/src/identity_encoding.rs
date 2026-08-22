use anyhow::{ensure, Context, Result};
use reqwest::header::{HeaderMap, CONTENT_ENCODING};

/// Requires response octets to use the identity representation requested by
/// byte-addressed media clients.
pub fn require_identity_encoding(headers: &HeaderMap) -> Result<()> {
    for value in headers.get_all(CONTENT_ENCODING) {
        let codings = value
            .to_str()
            .context("media Content-Encoding is not text")?;
        ensure!(
            codings
                .split(',')
                .all(|coding| coding.trim().eq_ignore_ascii_case("identity")),
            "media response has non-identity Content-Encoding"
        );
    }
    Ok(())
}
