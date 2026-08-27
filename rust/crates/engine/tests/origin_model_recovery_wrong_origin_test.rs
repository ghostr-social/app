use super::origin_model_recovery_fixture::{claim, open_circuit, query, query_at};
use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, OriginObservation, RequestMethod,
};

#[test]
fn wrong_origin_evidence_is_trained_but_cannot_settle_the_claim() {
    let full = query(RequestMethod::FullGet, 900_000);
    let other = query_at(
        "https://other.example/video.mp4",
        RequestMethod::RangeGet,
        65_536,
    );
    let mut model = open_circuit(&full);
    let active = claim(
        &mut model,
        &full,
        5_000,
        Admission::RecoveryProbe {
            maximum_bytes: 65_536,
        },
    );
    let before = model
        .estimate(&other, 5_101, DecisionMode::Normal)
        .effective_samples;
    let observed = OriginObservation::success(other.clone(), 5_100);

    assert!(!model.complete_claim(active, AdmissionClaimTerminal::Observed(&observed)));

    assert_eq!(
        model.circuit_admission(&full, 5_101),
        Admission::RecoveryProbe {
            maximum_bytes: 65_536
        }
    );
    assert!(
        model
            .estimate(&other, 5_101, DecisionMode::Normal)
            .effective_samples
            > before
    );
}
