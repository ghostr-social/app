use super::origin_model_recovery_fixture::{claim, open_circuit, query};
use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};

#[test]
fn stale_recovery_completion_cannot_resolve_a_newer_claim() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = open_circuit(&full);
    let expected = Admission::RecoveryProbe {
        maximum_bytes: 65_536,
    };
    let stale = claim(&mut model, &full, 5_000, expected);
    let current = claim(&mut model, &full, 35_000, expected);
    let range = query(RequestMethod::RangeGet, 65_536);
    let before = samples(&model, &range);
    let stale_result = OriginObservation::success(range.clone(), 35_050);
    let current_result = OriginObservation::success(range.clone(), 35_100);

    assert!(!model.complete_claim(stale, AdmissionClaimTerminal::Observed(&stale_result)));
    assert!(samples(&model, &range) > before);
    assert_eq!(model.circuit_admission(&full, 35_051), Admission::Blocked);
    assert!(model.complete_claim(current, AdmissionClaimTerminal::Observed(&current_result)));
    assert_eq!(
        model.circuit_admission(&full, 35_101),
        Admission::RecoveryTrial
    );
}

fn samples(model: &OriginModel, query: &OriginQuery) -> f64 {
    model
        .estimate(query, 35_101, DecisionMode::Normal)
        .effective_samples
}
