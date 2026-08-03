mod support;

use support::retry::assert_origin_response_is_retried;

#[tokio::test(start_paused = true)]
async fn retries_media_after_a_transient_origin_forbidden_response() {
    assert_origin_response_is_retried(
        b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        "ghostr-forbidden-retry",
    )
    .await;
}
