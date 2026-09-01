//! Metadata HEAD must complete without being counted as protected body IO.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::cooling_plan_origin::CoolingPlanOrigin;

const RESPONSE_LIMIT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn protected_head_probe_completes_with_valid_media_metadata() {
    let origin = CoolingPlanOrigin::serve().await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("valid test fixture");
    let request = client.head(origin.url("useful")).send();
    let response = tokio::time::timeout(RESPONSE_LIMIT, request)
        .await
        .expect("HEAD probe did not complete")
        .expect("HEAD response was disconnected");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.headers()[reqwest::header::CONTENT_LENGTH], "64");
    assert_eq!(response.headers()[reqwest::header::ACCEPT_RANGES], "bytes");
    assert_eq!(origin.useful_requests(), 0, "HEAD was counted as body IO");
}
