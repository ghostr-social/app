use super::asset;
use super::support::{client, immediate_status};
use crate::manager::failure::FailureClass;
use ghostr_engine::adaptive::PreemptionAuthority;
use reqwest::StatusCode;

#[tokio::test]
async fn http_status_preserves_origin_scoped_retry_classification() {
    for (raw, status, class) in [
        (
            "408 Request Timeout",
            StatusCode::REQUEST_TIMEOUT,
            FailureClass::Transient,
        ),
        (
            "429 Too Many Requests",
            StatusCode::TOO_MANY_REQUESTS,
            FailureClass::Transient,
        ),
        (
            "503 Service Unavailable",
            StatusCode::SERVICE_UNAVAILABLE,
            FailureClass::Transient,
        ),
        (
            "404 Not Found",
            StatusCode::NOT_FOUND,
            FailureClass::Permanent,
        ),
    ] {
        let (url, server) = immediate_status(raw).await;
        let url = url::Url::parse(&url).expect("valid test fixture");
        let failure = asset(&client(), &url, PreemptionAuthority::Transition)
            .await
            .err()
            .expect("HTTP status fails the HLS request");
        assert_eq!(failure.status(), Some(status));
        assert_eq!(failure.retry_class(), Some(class));
        server.await.expect("valid test fixture");
    }
}
