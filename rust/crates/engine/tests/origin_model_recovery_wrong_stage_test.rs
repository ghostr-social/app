use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{Admission, AdmissionClaimTerminal, OriginObservation, RequestMethod};

#[test]
fn probe_stage_full_success_cannot_bypass_the_exact_trial() {
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
    let unexpected = OriginObservation::success(full.clone(), 5_100);

    assert!(model.complete_claim(
        active,
        AdmissionClaimTerminal::ObservedWholeBody(&unexpected)
    ));

    assert_eq!(
        model.circuit_admission(&full, 5_101),
        Admission::RecoveryTrial
    );
}
