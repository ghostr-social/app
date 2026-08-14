mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::StatusCode;
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use ghostr_delivery::manager::DeliveryTuning;
use std::time::Duration;
use tower::ServiceExt;

#[tokio::test]
async fn lengthless_head_cannot_preempt_an_admitted_bootstrap_get() {
    let origin = ProgressiveJourneyOrigin::with_lengthless_head().await;
    let harness = ProgressiveDeliveryHarness::start_with_tuning(
        "ghostr-progressive-head-lengthless",
        fast_probe_retirement(),
    );
    harness.focus(vec![unknown_item("delivery-current", &origin.url)], 0);
    harness.wait_until_registered("delivery-current").await;
    let request = harness.request("delivery-current", "bytes=0-2047").await;

    let response = harness
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("gateway response");
    let status = response.status();
    let body = to_bytes(response.into_body(), 2_048).await.unwrap();

    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert!(!body.is_empty());
}

fn fast_probe_retirement() -> DeliveryTuning {
    let mut tuning = DeliveryTuning::default();
    tuning.retry.base = Duration::from_millis(10);
    tuning.retry.max = Duration::from_millis(10);
    tuning.retry.jitter = 0.0;
    tuning.retry.transient_attempts = 2;
    tuning
}
