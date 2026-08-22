use super::asset_fixture::optional_exchange;
use axum::body::to_bytes;
use axum::http::StatusCode;

const FULL: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nfull";

#[tokio::test]
async fn zero_length_suffix_is_rejected_without_contacting_the_asset_origin() {
    let exchange = optional_exchange(FULL.to_vec(), &["bytes=-0"]).await;
    let response = exchange.result.expect("local unsatisfiable response");

    assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(to_bytes(response.into_body(), 1).await.unwrap(), "");
    assert_eq!(exchange.requests.len(), 1);
}
