#[path = "queued_cancellation_test/fixture.rs"]
mod fixture;

#[tokio::test]
async fn cancellation_while_queued_does_not_count_an_http_request() {
    let result = fixture::cancel_queued_transfer().await;

    assert!(result.cancelled);
    assert!(!result.request_started);
}
