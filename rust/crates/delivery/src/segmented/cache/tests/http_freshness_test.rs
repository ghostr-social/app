use super::super::{CachedHlsObject, HlsCacheMetadata};
use reqwest::header::{HeaderMap, HeaderValue, CACHE_CONTROL, DATE, ETAG};
use std::sync::Arc;
use url::Url;

#[test]
fn old_date_consumes_the_explicit_freshness_lifetime() {
    let mut headers = headers("max-age=60");
    headers.insert(
        DATE,
        HeaderValue::from_static("Sun, 06 Nov 1994 08:49:37 GMT"),
    );

    assert!(!object(&headers).is_reusable());
}

#[test]
fn parameterized_no_cache_forbids_reuse() {
    assert!(!object(&headers("max-age=60, no-cache=\"set-cookie\"")).is_reusable());
}

#[test]
fn malformed_duplicate_max_age_forbids_reuse() {
    assert!(!object(&headers("max-age=60, max-age=invalid")).is_reusable());
}

#[test]
fn quoted_extension_cannot_manufacture_a_max_age_directive() {
    assert!(!object(&headers("extension=\"x,max-age=31536000\"")).is_reusable());
}

fn headers(cache_control: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    headers.insert(ETAG, HeaderValue::from_static("\"v1\""));
    headers
}

fn object(headers: &HeaderMap) -> CachedHlsObject {
    let url = Url::parse("https://media.example/index.m3u8").expect("valid test fixture");
    CachedHlsObject::with_metadata(
        Arc::from(b"#EXTM3U\n".as_slice()),
        url,
        None,
        HlsCacheMetadata::from_headers(headers),
    )
}
