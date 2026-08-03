mod support;

use support::retry::assert_origin_response_is_retried;

#[tokio::test(start_paused = true)]
async fn retries_an_origin_that_temporarily_returns_an_empty_success() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    assert_origin_response_is_retried(response, "ghostr-empty-origin-retry").await;
}
