use super::asset_sequence_origin::{header_values, request, request_error, serve};
use super::support::{asset_resource, state};
use axum::body::to_bytes;
use std::time::Duration;

macro_rules! response {
    ($range:literal, $etag:literal, $body:literal) => {
        concat!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\nContent-Range: bytes ",
            $range,
            "/16\r\nETag: \"",
            $etag,
            "\"\r\nConnection: close\r\n\r\n",
            $body
        )
        .as_bytes()
    };
}

const FIRST: &[u8] = response!("0-3", "v1", "aaaa");
const SECOND: &[u8] = response!("4-7", "v1", "bbbb");
const ROTATED_ETAG: &[u8] = response!("8-11", "v2", "cccc");
const ROTATED_TOTAL: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 8-11/20\r\nETag: \"v1\"\r\nConnection: close\r\n\r\ncccc";
const MISSING_ETAG: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 8-11/16\r\nConnection: close\r\n\r\ncccc";

#[tokio::test]
async fn one_hls_asset_keeps_one_origin_generation_for_every_range() {
    verify_rotation(ROTATED_ETAG).await;
    verify_rotation(ROTATED_TOTAL).await;
    verify_rotation(MISSING_ETAG).await;
}

async fn verify_rotation(rotated: &'static [u8]) {
    let (source, server) = serve(vec![FIRST, SECOND, rotated]).await;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;

    let first = request(&state, &session, &resource, "bytes=0-3").await;
    assert_eq!(to_bytes(first.into_body(), 4).await.unwrap(), "aaaa");
    let second = request(&state, &session, &resource, "bytes=4-7").await;
    assert_eq!(to_bytes(second.into_body(), 4).await.unwrap(), "bbbb");
    assert_eq!(
        request_error(&state, &session, &resource, "bytes=8-11").await,
        502
    );
    assert_eq!(
        request_error(&state, &session, &resource, "bytes=12-15").await,
        502
    );

    let requests = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("origin completion")
        .expect("origin task");
    assert_eq!(requests.len(), 3);
    assert_eq!(header_values(&requests[1], "if-range"), ["\"v1\""]);
    assert_eq!(header_values(&requests[2], "if-range"), ["\"v1\""]);
}
