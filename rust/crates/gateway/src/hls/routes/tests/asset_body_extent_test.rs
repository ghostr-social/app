use super::asset_fixture::{exchange, observed_body};

const UNDERFLOW: &[u8] = b"HTTP/1.1 206 Partial Content\r\nTransfer-Encoding: chunked\r\n\
Content-Range: bytes 4-7/16\r\nConnection: close\r\n\r\n2\r\nab\r\n0\r\n\r\n";
const OVERFLOW: &[u8] = b"HTTP/1.1 206 Partial Content\r\nTransfer-Encoding: chunked\r\n\
Content-Range: bytes 4-7/16\r\nConnection: close\r\n\r\n4\r\ngood\r\n1\r\nx\r\n0\r\n\r\n";

#[tokio::test]
async fn uncached_partial_body_must_match_its_declared_extent() {
    let underflow = exchange(UNDERFLOW.to_vec(), &["bytes=4-7"]).await;
    let (underflow_bytes, underflow_failed) = observed_body(underflow.result.unwrap()).await;
    assert!(underflow_failed);
    assert_eq!(underflow_bytes, 2);

    let overflow = exchange(OVERFLOW.to_vec(), &["bytes=4-7"]).await;
    let (overflow_bytes, overflow_failed) = observed_body(overflow.result.unwrap()).await;
    assert!(overflow_failed);
    assert_eq!(overflow_bytes, 4);
}
