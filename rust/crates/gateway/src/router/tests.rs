use super::upstream_request;
use axum::http::header::{ACCEPT_ENCODING, RANGE};
use axum::http::{HeaderMap, HeaderValue};
use ghostr_net::outbound_media_client::MediaHttpClient;

#[test]
fn hls_upstream_requests_preserve_byte_identity() {
    let client = MediaHttpClient::public().expect("guarded client");
    let mut headers = HeaderMap::new();
    headers.insert(RANGE, HeaderValue::from_static("bytes=4-9"));

    let request = upstream_request(
        &client,
        "https://media.example/video.ts".to_owned(),
        &headers,
    )
    .expect("request")
    .build()
    .expect("built request");

    assert_eq!(request.headers()[RANGE], "bytes=4-9");
    assert_eq!(request.headers()[ACCEPT_ENCODING], "identity");
}
