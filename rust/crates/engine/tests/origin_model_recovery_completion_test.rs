use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, ErrorReason, MediaClass,
    OriginAdmissionIntent, OriginContext, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};

#[test]
fn full_get_recovery_closes_only_after_network_eof() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = open_circuit(&full);
    let probe = claim(&mut model, &full, 5_000);
    let range = OriginObservation::success(query(RequestMethod::RangeGet, 65_536), 5_100);
    model.complete_claim(probe, AdmissionClaimTerminal::Observed(&range));
    let first_trial = claim(&mut model, &full, 5_101);
    let incomplete = OriginObservation::success(full.clone(), 5_200);

    model.complete_claim(first_trial, AdmissionClaimTerminal::Observed(&incomplete));

    assert_eq!(model.circuit_admission(&full, 5_201), Admission::Blocked);
    let final_trial = claim(&mut model, &full, 10_000);
    let complete = OriginObservation::success(full.clone(), 10_100);
    model.complete_claim(
        final_trial,
        AdmissionClaimTerminal::ObservedWholeBody(&complete),
    );
    assert_eq!(
        model.circuit_admission(&full, 10_101),
        Admission::Production
    );
}

fn query(method: RequestMethod, bytes: u64) -> OriginQuery {
    OriginQuery::new(
        "https://recovered.example/video.mp4",
        OriginContext::new(method, bytes, MediaClass::WholeObject),
    )
}

fn open_circuit(query: &OriginQuery) -> OriginModel {
    let mut model = OriginModel::default();
    for at_ms in 1_000..=1_002 {
        model.observe(&OriginObservation::failure(
            query.clone(),
            at_ms,
            ErrorReason::Timeout,
        ));
    }
    model
}

fn claim(
    model: &mut OriginModel,
    query: &OriginQuery,
    at_ms: u64,
) -> crate::origin_model::AdmissionClaim {
    model
        .claim(
            query,
            at_ms,
            DecisionMode::Normal,
            OriginAdmissionIntent::Delivery,
        )
        .into_parts()
        .1
        .expect("recovery claim")
}
