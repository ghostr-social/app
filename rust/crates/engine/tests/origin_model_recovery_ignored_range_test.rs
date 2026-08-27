use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{Admission, AdmissionClaimTerminal, OriginObservation, RequestMethod};

#[test]
fn ignored_range_response_permits_only_one_exact_full_trial() {
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
    let ignored = OriginObservation::success(query(RequestMethod::RangeGet, 65_536), 5_100)
        .with_range_compliance(false);

    assert!(model.complete_claim(active, AdmissionClaimTerminal::Observed(&ignored)));

    assert_eq!(
        model.circuit_admission(&full, 5_101),
        Admission::RecoveryTrial
    );
}
