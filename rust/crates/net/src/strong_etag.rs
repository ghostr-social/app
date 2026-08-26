//! Exact singleton strong-ETag admission for byte-generation identity.

use reqwest::header::{HeaderMap, HeaderValue, ETAG};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrongEtag(HeaderValue);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidStrongEtag;

/// # Errors
///
/// Returns an error when the response contains multiple, weak, or malformed `ETag` values.
pub fn single_strong_etag(headers: &HeaderMap) -> Result<Option<StrongEtag>, InvalidStrongEtag> {
    let mut values = headers.get_all(ETAG).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(InvalidStrongEtag);
    }
    valid_opaque_tag(value.as_bytes())
        .then(|| Some(StrongEtag(value.clone())))
        .ok_or(InvalidStrongEtag)
}

impl StrongEtag {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_header_value(&self) -> &HeaderValue {
        &self.0
    }

    pub fn to_ascii(&self) -> Option<&str> {
        self.0.to_str().ok()
    }
}

fn valid_opaque_tag(value: &[u8]) -> bool {
    let Some(inner) = value
        .strip_prefix(b"\"")
        .and_then(|tag| tag.strip_suffix(b"\""))
    else {
        return false;
    };
    inner
        .iter()
        .all(|byte| *byte == 0x21 || (0x23..=0x7e).contains(byte) || *byte >= 0x80)
}
