use super::{readiness_event_for, reserve_underflow};
use ghostr_engine::adaptive::{
    ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState,
};
use ghostr_engine::media_timeline::StartupFootprint;
use ghostr_engine::PostId;

#[test]
fn a_ready_item_beyond_a_gap_is_reported_as_reserve_underflow() {
    assert!(reserve_underflow(&gapped_reserve()));
}

#[test]
fn a_plan_snapshot_does_not_claim_a_future_calibration_outcome() {
    let event = readiness_event_for(&gapped_reserve(), false, 42);

    assert_eq!(event.on_time_prediction_bps, None);
    assert_eq!(event.on_time_observed, None);
}

fn gapped_reserve() -> ReadyReserveEvidence {
    let startup = startup();
    ReadyReserveEvidence {
        target: 3,
        ready: 3,
        candidates: vec![
            candidate(
                "p1",
                ReserveCandidateState::Ready {
                    startup: startup.clone(),
                },
            ),
            candidate(
                "p2",
                ReserveCandidateState::Ready {
                    startup: startup.clone(),
                },
            ),
            candidate(
                "p3",
                ReserveCandidateState::Structural {
                    startup: startup.clone(),
                },
            ),
            candidate("p4", ReserveCandidateState::Ready { startup }),
        ],
        ..ReadyReserveEvidence::default()
    }
}

fn startup() -> StartupFootprint {
    serde_json::from_value(serde_json::json!({
        "ranges": [{"start": 0, "end": 64}],
        "playable_ms": 2_000,
        "provenance": 1
    }))
    .expect("valid startup fixture")
}

fn candidate(post: &str, state: ReserveCandidateState) -> ReserveCandidateEvidence {
    ReserveCandidateEvidence {
        post: PostId::new(post),
        kind: Default::default(),
        state,
    }
}
