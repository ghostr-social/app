use super::asset_fixture::{exchange, range_values};
use axum::body::to_bytes;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE};
use axum::http::StatusCode;

const WRONG: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 0-3/16\r\nConnection: close\r\n\r\nbad!";
const COHERENT: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 4-7/16\r\nConnection: close\r\n\r\ngood";
const SIGNED: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes +4-+7/+16\r\nConnection: close\r\n\r\ngood";

#[tokio::test]
async fn uncached_asset_requires_the_exact_requested_response_range() {
    let wrong = exchange(WRONG.to_vec(), &["bytes=4-7"]).await;
    assert_eq!(
        wrong.result.expect_err("wrong range"),
        StatusCode::BAD_GATEWAY
    );
    assert_eq!(range_values(&wrong.requests[1]), ["bytes=4-7"]);

    let signed = exchange(SIGNED.to_vec(), &["bytes=4-7"]).await;
    assert_eq!(
        signed.result.expect_err("signed response range"),
        StatusCode::BAD_GATEWAY
    );

    let valid = exchange(COHERENT.to_vec(), &["bytes=4-7"]).await;
    let response = valid.result.expect("coherent range");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(response.headers()[CONTENT_RANGE], "bytes 4-7/16");
    assert_eq!(response.headers()[CONTENT_LENGTH], "4");
    assert_eq!(response.headers()[ACCEPT_RANGES], "bytes");
    assert_eq!(
        to_bytes(response.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "good"
    );
}
