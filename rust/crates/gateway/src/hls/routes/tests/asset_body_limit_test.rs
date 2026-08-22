use super::asset_fixture::exchange;
use axum::http::StatusCode;

const MAX_HLS_ASSET_BYTES: usize = 8 * 1024 * 1024;

#[tokio::test]
async fn oversized_uncached_asset_is_rejected_before_body_exposure() {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        MAX_HLS_ASSET_BYTES + 1
    );
    let exchange = exchange(response.into_bytes(), &[]).await;

    assert_eq!(
        exchange.result.expect_err("oversized asset"),
        StatusCode::BAD_GATEWAY
    );
}

#[tokio::test]
async fn oversized_partial_extent_is_rejected_before_body_exposure() {
    let response = format!(
        "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-{0}/*\r\n\
         Content-Length: {1}\r\nConnection: close\r\n\r\n",
        MAX_HLS_ASSET_BYTES,
        MAX_HLS_ASSET_BYTES + 1
    );
    let exchange = exchange(response.into_bytes(), &["bytes=0-99999999"]).await;

    assert_eq!(
        exchange.result.expect_err("oversized partial"),
        StatusCode::BAD_GATEWAY
    );
}
