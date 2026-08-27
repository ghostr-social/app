use super::origin_claim::settle;
use crate::manager::transfers::ChunkDone;
use ghostr_engine::origin_model::{Admission, DecisionMode, OriginObservation, RequestMethod};

#[path = "origin_claim_fixture.rs"]
mod fixture;
use fixture::{finished_action, open_circuit, query, success, URL};

#[test]
fn physical_range_terminal_advances_its_full_get_recovery_claim() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = open_circuit(&full);
    let claimed = model.claim(&full, 5_000, DecisionMode::Normal);
    assert!(matches!(
        claimed.admission(),
        Admission::RecoveryProbe { .. }
    ));
    let (_, claim) = claimed.into_parts();
    let (attempt, mut finished) = finished_action(claim.expect("recovery claim"));
    let physical = OriginObservation::success(query(RequestMethod::RangeGet, 65_536), 5_100);
    let done = ChunkDone {
        attempt,
        url: URL.into(),
        outcome: Ok(success()),
        received_bytes: 65_536,
        origin: Some(Box::new(physical)),
        request_started: true,
        whole_body_completion: None,
        response_evidence: None,
    };

    assert!(settle(&mut model, &done, &mut finished));
    assert!(!settle(&mut model, &done, &mut finished));

    assert_eq!(
        model.circuit_admission(&full, 5_101),
        Admission::RecoveryTrial
    );
}
