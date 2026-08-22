#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use gateway_fixture::ready_reserve::plan;
use tower::ServiceExt;

#[tokio::test]
async fn debug_state_exposes_the_rolling_ready_reserve() {
    let mut harness = progressive_harness("debug-ready-reserve");
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

    assert_eq!(evidence["mode"], "safety");
    assert_eq!(
        evidence["ready_reserve"],
        serde_json::json!({
            "target": 3,
            "ready": 1,
            "structural": 1,
            "protected": 3,
            "recovery_horizon_ms": 1800,
            "underflow_risk_bps": 420,
            "ready_coverage_ms": 2300,
            "candidates": [
                {"post_id": "p1", "status": "ready"},
                {"post_id": "p2", "status": "structural"},
                {"post_id": "p3", "status": "planned", "ranges": [{"start": 0, "end": 8}]},
                {"post_id": "p4", "status": "infeasible", "reason": "no_live_origin"}
            ]
        })
    );
}
