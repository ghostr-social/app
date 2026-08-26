mod gateway_fixture;

use axum::http::Method;
use core::time::Duration;
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use ghostr_delivery::manager::DeliveryTuning;

#[tokio::test]
async fn deferred_probe_retries_after_the_active_body_fails() {
    let origin = ProgressiveJourneyOrigin::with_deferred_probe_and_failed_body().await;
    let harness =
        ProgressiveDeliveryHarness::start_with_tuning("ghostr-deferred-probe-retry", fast_retry());
    harness.focus(vec![unknown_item("delivery-current", &origin.url)], 0);

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let heads = origin
                .requests()
                .iter()
                .filter(|request| request.method == Method::HEAD)
                .count();
            if heads >= 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("failed deferred body releases the probe for retry");
}

fn fast_retry() -> DeliveryTuning {
    let mut tuning = DeliveryTuning::default();
    tuning.retry.base = Duration::from_millis(10);
    tuning.retry.max = Duration::from_millis(10);
    tuning.retry.jitter = 0.0;
    tuning.retry.transient_attempts = 4;
    tuning
}
