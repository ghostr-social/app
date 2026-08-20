#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::adaptive_plan::plan;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn debug_state_exposes_exact_adaptive_plan_evidence() {
    let mut harness = progressive_harness("debug-adaptive-plan");
    harness.debug_commands.publish_plan(42, plan());
    let request = Request::get("/debug/api/state")
        .body(Body::empty())
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state");
    let evidence = &state["adaptive_plans"][0];

    assert_eq!(evidence["revision"], 1);
    assert_eq!(evidence["observed_at_ms"], 42);
    assert_eq!(evidence["discovery_demand"], "hold");
    assert_eq!(evidence["allocations"][0]["post_id"], "p1");
    assert_eq!(
        evidence["allocations"][0]["range"],
        serde_json::json!({"start": 10, "end": 20})
    );
    assert_eq!(evidence["allocations"][0]["reason"], "media_bootstrap");
    assert_eq!(evidence["allocations"][0]["expected_playable_gain_ms"], 500);
    assert_eq!(
        evidence["retained"][0]["utility"]["expected_delivery_ms"],
        20
    );
    assert_eq!(
        evidence["next_reserve"],
        serde_json::json!({
            "status": "infeasible",
            "post_id": "p2",
            "reason": "no_transfer_budget"
        })
    );
}
