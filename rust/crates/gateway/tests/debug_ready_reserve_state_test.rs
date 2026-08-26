#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::adaptive::{
    AllocationPlan, ControlMode, NextReserveInfeasibility, ReadyReserveEvidence,
    ReserveCandidateEvidence, ReserveCandidateState,
};
use ghostr_engine::{ByteRange, PostId};
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
            "structural": 0,
            "protected": 2,
            "recovery_horizon_ms": 1800,
            "underflow_risk_bps": 420,
            "ready_coverage_ms": 2300,
            "candidates": [
                {"post_id": "p1", "status": "ready"},
                {"post_id": "p2", "status": "planned", "ranges": [{"start": 0, "end": 8}]},
                {"post_id": "p3", "status": "infeasible", "reason": "no_live_origin"}
            ]
        })
    );
}

fn plan() -> AllocationPlan {
    AllocationPlan {
        mode: ControlMode::Safety,
        ready_reserve: ReadyReserveEvidence {
            target: 3,
            ready: 1,
            structural: 0,
            protected: 2,
            recovery_horizon_ms: 1_800,
            underflow_risk_bps: 420,
            ready_coverage_ms: 2_300,
            candidates: vec![ready(), planned(), unavailable()],
        },
        ..AllocationPlan::default()
    }
}

fn ready() -> ReserveCandidateEvidence {
    evidence(
        "p1",
        ReserveCandidateState::Ready {
            startup: gateway_fixture::progressive_startup(),
        },
    )
}

fn planned() -> ReserveCandidateEvidence {
    evidence(
        "p2",
        ReserveCandidateState::Planned {
            ranges: vec![ByteRange::new(0, 8)],
        },
    )
}

fn unavailable() -> ReserveCandidateEvidence {
    evidence(
        "p3",
        ReserveCandidateState::Infeasible {
            reason: NextReserveInfeasibility::NoLiveOrigin,
        },
    )
}

fn evidence(post: &str, state: ReserveCandidateState) -> ReserveCandidateEvidence {
    ReserveCandidateEvidence {
        post: PostId::new(post),
        state,
    }
}
