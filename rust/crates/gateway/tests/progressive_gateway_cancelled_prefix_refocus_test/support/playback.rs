use crate::support::{
    focus_trimmed_and_wait, next_prefix, wait_for_blocked, wait_for_current_authority,
    CancelledPrefixScenario,
};
use axum::body::to_bytes;
use axum::http::StatusCode;
use tower::ServiceExt as _;

impl CancelledPrefixScenario {
    pub async fn expect_first_active_demand_refill(&mut self) {
        focus_trimmed_and_wait(&self.harness, &self.items, 6, 13).await;
        wait_for_current_authority(&self.harness, "p6").await;
        let request = self.harness.request("p6", "bytes=0-65535").await;
        let response = self
            .harness
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("gateway response");
        assert_active_authority(&self.harness, response.status());
        let body = tokio::spawn(to_bytes(response.into_body(), 65_536));
        wait_for_blocked(&self.harness, None, "first active p6 gateway demand").await;
        let replacement = next_prefix(&mut self.origin, &self.harness).await;
        assert!(replacement.send_bytes(65_536).await);
        let body = body.await.expect("body task").expect("gateway body");
        assert_eq!(body.as_ref(), &self.bytes[..65_536]);
    }
}

fn assert_active_authority(
    harness: &crate::gateway_fixture::progressive_delivery::ProgressiveDeliveryHarness,
    status: StatusCode,
) {
    assert_eq!(
        status,
        StatusCode::PARTIAL_CONTENT,
        "active p6 authority rejected; latest_plan={:#?}",
        harness.delivery.handle.latest_plan(),
    );
}
