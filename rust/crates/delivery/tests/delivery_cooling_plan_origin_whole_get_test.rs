//! The cooling-plan origin must support the whole-body GET selected by policy.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::cooling_plan_origin::CoolingPlanOrigin;

const WAIT_LIMIT: Duration = Duration::from_secs(30);

#[tokio::test]
async fn protected_whole_get_completes_after_release() {
    let origin = CoolingPlanOrigin::serve().await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("valid test fixture");
    let url = origin.url("useful");
    let request = tokio::spawn(async move { client.get(url).send().await });

    tokio::time::timeout(WAIT_LIMIT, origin.wait_useful())
        .await
        .expect("whole GET did not reach the fixture");
    origin.release();
    let response = tokio::time::timeout(WAIT_LIMIT, request)
        .await
        .expect("whole GET did not complete")
        .expect("whole GET task panicked")
        .expect("whole GET response was disconnected");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.bytes().await.expect("whole body").len(), 64);
    assert_eq!(origin.useful_requests(), 1);
}
