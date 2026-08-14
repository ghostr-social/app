#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::adaptive::{AllocationPlan, NextReserveEvidence};
use ghostr_engine::PostId;
use tower::ServiceExt;

#[tokio::test]
async fn debug_state_distinguishes_inflight_next_from_ready() {
    let mut harness = progressive_harness("debug-adaptive-inflight");
    harness.debug_commands.publish_plan(
        42,
        AllocationPlan {
            next_reserve: NextReserveEvidence::InFlight {
                post: PostId::new("p2"),
            },
            ..AllocationPlan::default()
        },
    );
    let request = Request::get("/debug/api/state")
        .body(Body::empty())
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state");

    assert_eq!(
        state["adaptive_plans"][0]["next_reserve"],
        serde_json::json!({"status": "in_flight", "post_id": "p2"})
    );
}
