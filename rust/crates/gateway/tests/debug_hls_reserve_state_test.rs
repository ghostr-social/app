#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::adaptive::{
    AllocationPlan, HlsBootstrapStage, NextReserveEvidence, ReadyReserveEvidence,
    ReserveCandidateEvidence, ReserveCandidateKind, ReserveCandidateState,
};
use ghostr_engine::PostId;
use tower::ServiceExt;

#[tokio::test]
async fn debug_state_preserves_hls_reserve_lifecycle() {
    let mut harness = progressive_harness("debug-hls-reserve");
    harness.debug_commands.publish_plan(42, plan());
    let request = Request::get("/debug/api/state")
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: serde_json::Value = serde_json::from_slice(&body).expect("state");
    let plan = &state["adaptive_plans"][0];

    assert_eq!(
        plan["ready_reserve"]["candidates"],
        serde_json::json!([
            {"post_id": "p1", "status": "hls_ready"},
            {"post_id": "p2", "status": "hls_structural"},
            {"post_id": "p3", "status": "hls_in_flight", "stage": "child_playlist"},
            {"post_id": "p4", "status": "hls_pending", "stage": "first_segment"}
        ])
    );
    assert_eq!(
        plan["next_reserve"],
        serde_json::json!({
            "status": "hls_pending",
            "post_id": "p4",
            "stage": "root_manifest"
        })
    );
}

fn plan() -> AllocationPlan {
    AllocationPlan {
        ready_reserve: ReadyReserveEvidence {
            target: 4,
            candidates: vec![
                evidence("p1", ReserveCandidateState::HlsReady),
                evidence("p2", ReserveCandidateState::HlsStructural),
                evidence(
                    "p3",
                    ReserveCandidateState::HlsInFlight {
                        stage: HlsBootstrapStage::ChildPlaylist,
                    },
                ),
                evidence(
                    "p4",
                    ReserveCandidateState::HlsPending {
                        stage: HlsBootstrapStage::FirstSegment,
                    },
                ),
            ],
            ..ReadyReserveEvidence::default()
        },
        next_reserve: NextReserveEvidence::HlsPending {
            post: PostId::new("p4"),
            stage: HlsBootstrapStage::RootManifest,
        },
        ..AllocationPlan::default()
    }
}

fn evidence(post: &str, state: ReserveCandidateState) -> ReserveCandidateEvidence {
    ReserveCandidateEvidence {
        post: PostId::new(post),
        kind: ReserveCandidateKind::Hls,
        state,
    }
}
