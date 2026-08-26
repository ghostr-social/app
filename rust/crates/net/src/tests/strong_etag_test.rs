use crate::strong_etag::single_strong_etag;
use reqwest::header::{HeaderMap, HeaderValue, ETAG};

#[test]
fn only_one_well_formed_strong_etag_is_identity_evidence() {
    assert_eq!(etag_bytes(&headers(&["\"v1\""])), Some(b"\"v1\"".to_vec()));
    assert_eq!(etag_bytes(&headers(&["\"\""])), Some(b"\"\"".to_vec()));
    assert_eq!(
        single_strong_etag(&HeaderMap::new()).expect("valid test fixture"),
        None
    );
    assert!(single_strong_etag(&headers(&["W/\"v1\""])).is_err());
    assert!(single_strong_etag(&headers(&["\"a\"", "\"b\""])).is_err());
    assert!(single_strong_etag(&headers(&["\"a\", \"b\""])).is_err());
    assert!(single_strong_etag(&headers(&["\"bad value\""])).is_err());
    assert!(single_strong_etag(&headers(&["\"bad\"quote\""])).is_err());
    assert!(single_strong_etag(&headers(&[" \"v1\" "])).is_err());
    let mut opaque = HeaderMap::new();
    opaque.insert(
        ETAG,
        HeaderValue::from_bytes(&[b'"', 0x80, b'"']).expect("valid test fixture"),
    );
    assert_eq!(etag_bytes(&opaque), Some(vec![b'"', 0x80, b'"']));
}

fn etag_bytes(headers: &HeaderMap) -> Option<Vec<u8>> {
    single_strong_etag(headers)
        .ok()
        .flatten()
        .map(|etag| etag.as_bytes().to_owned())
}

fn headers(values: &[&str]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for value in values {
        headers.append(ETAG, HeaderValue::from_str(value).expect("ETag fixture"));
    }
    headers
}
