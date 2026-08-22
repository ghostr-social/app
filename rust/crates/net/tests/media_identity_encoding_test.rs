use ghostr_net::identity_encoding::require_identity_encoding;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_ENCODING};

#[test]
fn identity_encoding_accepts_only_absent_or_identity_tokens() {
    assert!(require_identity_encoding(&HeaderMap::new()).is_ok());
    assert!(require_identity_encoding(&headers(&[b"identity", b" IDENTITY , identity ",])).is_ok());
}

#[test]
fn identity_encoding_rejects_every_ambiguous_or_coded_value() {
    for values in [
        vec![b"identity".as_slice(), b"gzip".as_slice()],
        vec![b"identity, gzip".as_slice()],
        vec![b"".as_slice()],
        vec![b"identity,".as_slice()],
        vec![b"identity; q=1".as_slice()],
        vec![b"\xff".as_slice()],
    ] {
        assert!(require_identity_encoding(&headers(&values)).is_err());
    }
}

fn headers(values: &[&[u8]]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for value in values {
        headers.append(
            CONTENT_ENCODING,
            HeaderValue::from_bytes(value).expect("header value"),
        );
    }
    headers
}
