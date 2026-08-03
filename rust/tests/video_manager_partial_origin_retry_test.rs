mod support;

use support::retry::assert_origin_response_is_retried;

#[tokio::test(start_paused = true)]
async fn retries_an_origin_that_temporarily_returns_partial_content() {
    let response = b"HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\n\r\nvid";
    assert_origin_response_is_retried(response, "ghostr-partial-origin-retry").await;
}
