use super::asset_fixture::{exchange, range_values};
use axum::body::to_bytes;
use axum::http::StatusCode;

const FULL: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nfull";
const PARTIAL: &[u8] = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 4\r\n\
Content-Range: bytes 4-7/16\r\nConnection: close\r\n\r\ngood";

#[tokio::test]
async fn range_collector_ignores_invalid_sets_and_canonicalizes_one_valid_spec() {
    let duplicate = exchange(FULL.to_vec(), &["bytes=4-7", "bytes=8-11"]).await;
    assert!(range_values(&duplicate.requests[1]).is_empty());
    assert_eq!(
        duplicate.result.expect("ignored duplicate").status(),
        StatusCode::OK
    );

    for value in ["bytes=5-2", "bytes=0-1,3-4", "bytes=+4-+7", "items=0-3"] {
        let ignored = exchange(FULL.to_vec(), &[value]).await;
        assert!(range_values(&ignored.requests[1]).is_empty(), "{value}");
        assert_eq!(
            ignored.result.expect("ignored range").status(),
            StatusCode::OK
        );
    }

    let valid = exchange(PARTIAL.to_vec(), &["BYTES=4-7"]).await;
    assert_eq!(range_values(&valid.requests[1]), ["bytes=4-7"]);
    let response = valid.result.expect("canonical range");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        to_bytes(response.into_body(), 4)
            .await
            .expect("valid test fixture"),
        "good"
    );
}
