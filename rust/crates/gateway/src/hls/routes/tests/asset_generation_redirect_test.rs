use super::asset_sequence_origin::{header_values, request, request_error, serve};
use super::support::{asset_resource, state};
use axum::body::to_bytes;
use std::time::Duration;

const REDIRECT_A: &[u8] = b"HTTP/1.1 302 Found\r\nLocation: /a.m4s\r\n\
Content-Length: 0\r\nConnection: close\r\n\r\n";
const REDIRECT_B: &[u8] = b"HTTP/1.1 302 Found\r\nLocation: /b.m4s\r\n\
Content-Length: 0\r\nConnection: close\r\n\r\n";
const FIRST: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 0-3/16\r\nETag: \"v1\"\r\nConnection: close\r\n\r\naaaa";
const RETARGETED: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 4-7/16\r\nETag: \"v1\"\r\nConnection: close\r\n\r\nbbbb";

#[tokio::test]
async fn redirected_hls_asset_cannot_change_its_final_url_generation() {
    let (source, server) = serve(vec![REDIRECT_A, FIRST, REDIRECT_B, RETARGETED]).await;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;

    let first = request(&state, &session, &resource, "bytes=0-3").await;
    assert_eq!(to_bytes(first.into_body(), 4).await.unwrap(), "aaaa");
    assert_eq!(
        request_error(&state, &session, &resource, "bytes=4-7").await,
        502
    );
    assert_eq!(
        request_error(&state, &session, &resource, "bytes=8-11").await,
        502
    );

    let requests = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("origin completion")
        .expect("origin task");
    assert_eq!(requests.len(), 4);
    assert!(requests[0].starts_with("GET /segment.m4s "));
    assert!(requests[1].starts_with("GET /a.m4s "));
    assert!(requests[2].starts_with("GET /segment.m4s "));
    assert!(requests[3].starts_with("GET /b.m4s "));
    assert_eq!(header_values(&requests[2], "if-range"), ["\"v1\""]);
    assert_eq!(header_values(&requests[3], "if-range"), ["\"v1\""]);
}
