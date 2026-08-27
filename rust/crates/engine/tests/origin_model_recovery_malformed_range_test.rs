use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{
    Admission, AdmissionClaimTerminal, ErrorReason, OriginObservation, RequestMethod,
};

#[test]
fn malformed_range_failure_reopens_recovery_with_backoff() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = open_circuit(&full);
    let active = claim(
        &mut model,
        &full,
        5_000,
        Admission::RecoveryProbe {
            maximum_bytes: 65_536,
        },
    );
    let malformed = OriginObservation::failure(
        query(RequestMethod::RangeGet, 65_536),
        5_100,
        ErrorReason::RangeNoncompliant,
    );

    assert!(model.complete_claim(active, AdmissionClaimTerminal::Observed(&malformed)));

    assert_eq!(model.circuit_admission(&full, 5_101), Admission::Blocked);
}
