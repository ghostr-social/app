use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{
    Admission, AdmissionBlockReason, AdmissionClaimTerminal, DecisionMode, OriginAdmissionIntent,
    OriginObservation, RequestMethod,
};

#[test]
fn sparse_success_requires_one_exact_full_get_recovery_trial() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = open_circuit(&full);
    let probe = claim(
        &mut model,
        &full,
        5_000,
        Admission::RecoveryProbe {
            maximum_bytes: 65_536,
        },
    );
    let physical = OriginObservation::success(query(RequestMethod::RangeGet, 65_536), 5_100);

    assert!(model.complete_claim(probe, AdmissionClaimTerminal::Observed(&physical)));
    assert_eq!(
        model.circuit_admission(&full, 5_101),
        Admission::RecoveryTrial
    );

    let trial = claim(&mut model, &full, 5_102, Admission::RecoveryTrial);
    let held = model.claim(
        &full,
        5_103,
        DecisionMode::Normal,
        OriginAdmissionIntent::Delivery,
    );
    assert_eq!(held.admission(), Admission::Blocked);
    assert_eq!(
        held.block_reason(),
        Some(AdmissionBlockReason::RecoveryLease)
    );
    let exact = OriginObservation::success(full.clone(), 5_200);
    assert!(model.complete_claim(trial, AdmissionClaimTerminal::ObservedWholeBody(&exact)));
    assert_eq!(model.circuit_admission(&full, 5_201), Admission::Production);
}
