use ghostr_net::response_limits::{validate_response_headers, MAX_MEDIA_RESPONSE_HEADER_BYTES};
use reqwest::header::{HeaderMap, HeaderValue};

#[test]
fn media_response_headers_have_a_hard_aggregate_limit() {
    let mut ordinary = HeaderMap::new();
    ordinary.insert("content-type", HeaderValue::from_static("video/mp4"));
    assert!(validate_response_headers(&ordinary).is_ok());

    let mut oversized = HeaderMap::new();
    let value = HeaderValue::from_bytes(&vec![b'a'; MAX_MEDIA_RESPONSE_HEADER_BYTES])
        .expect("valid header bytes");
    oversized.insert("x-padding", value);
    assert!(validate_response_headers(&oversized).is_err());
}
