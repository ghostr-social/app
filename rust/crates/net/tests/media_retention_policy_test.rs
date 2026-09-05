use ghostr_net::media_retention::MediaRetention;
use reqwest::header::{HeaderMap, HeaderValue, CACHE_CONTROL, SET_COOKIE, VARY};

#[test]
fn derived_reuse_requires_unambiguous_public_access_and_retention() {
    let url = reqwest::Url::parse("https://media.example/clip").expect("fixture");
    let mut headers = HeaderMap::new();
    assert_eq!(
        MediaRetention::from_headers(&headers, &url),
        MediaRetention::Partitioned
    );
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=60"),
    );
    assert_eq!(
        MediaRetention::from_headers(&headers, &url),
        MediaRetention::Public
    );
    for value in [
        "public, no-store",
        "PUBLIC, NO-STORE",
        "public, no-cache",
        "private=\"ETag\"",
        "public, extension=\"x,no-store\"",
    ] {
        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_str(value).expect("fixture"),
        );
        assert_eq!(
            MediaRetention::from_headers(&headers, &url),
            MediaRetention::Transient
        );
    }
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("public, private"));
    assert_eq!(
        MediaRetention::from_headers(&headers, &url),
        MediaRetention::Transient
    );
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("public"));
    headers.append(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    assert_eq!(
        MediaRetention::from_headers(&headers, &url),
        MediaRetention::Transient
    );
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("public"));
    for name in [VARY, SET_COOKIE] {
        headers.insert(name.clone(), HeaderValue::from_static("session"));
        assert_eq!(
            MediaRetention::from_headers(&headers, &url),
            MediaRetention::Transient
        );
        headers.remove(name);
    }
    let signed = reqwest::Url::parse("https://media.example/clip?token=private").expect("fixture");
    assert_eq!(
        MediaRetention::from_headers(&headers, &signed),
        MediaRetention::Transient
    );
}
