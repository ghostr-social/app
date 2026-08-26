use super::asset_sequence_origin::{request, request_error, serve};
use super::support::{asset_resource, state};
use axum::body::to_bytes;
use core::time::Duration;

const NO_ETAG: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 0-3/16\r\nConnection: close\r\n\r\nonce";
const NO_TOTAL: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 0-3/*\r\nETag: \"v1\"\r\nConnection: close\r\n\r\nonce";

#[tokio::test]
async fn every_incomplete_hls_generation_is_delivered_once_then_retired() {
    for response in [NO_ETAG, NO_TOTAL] {
        verify_one_shot(response).await;
    }
}

async fn verify_one_shot(response: &'static [u8]) {
    let (source, server) = serve(vec![response]).await;
    let (state, session) = state(source).await;
    let resource = asset_resource(&state, &session).await;

    let first = request(&state, &session, &resource, "bytes=0-3").await;
    assert_eq!(
        to_bytes(first.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "once"
    );
    assert_eq!(
        request_error(&state, &session, &resource, "bytes=4-7").await,
        502
    );

    let requests = tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("origin completion")
        .expect("origin task");
    assert_eq!(requests.len(), 1);
}
