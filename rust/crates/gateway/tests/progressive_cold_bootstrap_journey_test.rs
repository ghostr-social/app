mod gateway_fixture;

use axum::body::to_bytes;
use axum::http::{Method, StatusCode};
use core::time::Duration;
use gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness;
use gateway_fixture::progressive_journey_item::unknown_item;
use gateway_fixture::progressive_journey_origin::ProgressiveJourneyOrigin;
use tower::ServiceExt as _;

#[tokio::test]
async fn cold_focus_starts_admitted_body_bytes_without_waiting_for_head() {
    let origin = ProgressiveJourneyOrigin::with_blocked_head().await;
    let harness = ProgressiveDeliveryHarness::start("ghostr-progressive-bootstrap");
    harness.focus(vec![unknown_item("delivery-current", &origin.url)], 0);
    harness.wait_until_registered("delivery-current").await;
    let request = harness.request("delivery-current", "bytes=0-2047").await;

    let response = tokio::time::timeout(
        Duration::from_secs(3),
        harness.router.clone().oneshot(request),
    )
    .await
    .expect("gateway response without HEAD completion")
    .expect("gateway response");
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    let body = to_bytes(response.into_body(), 2_048)
        .await
        .expect("body bytes");
    assert!(
        !body.is_empty(),
        "the first gateway response carries useful MP4 bytes"
    );

    let requests = origin.requests();
    assert!(requests.iter().any(|request| request.method == Method::GET));
    assert!(
        harness.delivery.handle.plan_history().iter().any(|entry| {
            entry
                .plan
                .allocations
                .iter()
                .any(|allocation| allocation.post.as_str() == "delivery-current")
        }),
        "the useful body request has prior policy admission"
    );
    for range in origin.get_ranges() {
        assert!(
            harness.delivery.handle.plan_history().iter().any(|entry| {
                entry.plan.allocations.iter().any(|allocation| {
                    let requested = allocation.request.requested_bytes();
                    allocation.post.as_str() == "delivery-current"
                        && requested.start == range.start
                        && requested.end == range.end
                })
            }),
            "origin range {range:?} must exactly match prior plan evidence"
        );
    }
    let stored = harness
        .delivery
        .store
        .present_ranges("delivery-current")
        .await
        .expect("stored ranges");
    assert!(!stored.is_empty(), "the bootstrap prefix is retained");
    assert_eq!(origin.total_bytes(), 293_999);
}
