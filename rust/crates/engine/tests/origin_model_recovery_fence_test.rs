use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, ErrorReason, MediaClass, OriginContext,
    OriginModel, OriginObservation, OriginQuery, RequestMethod,
};

#[test]
fn an_unclaimed_late_success_cannot_settle_a_recovery_lease() {
    let full = query(RequestMethod::FullGet, 900_000);
    let mut model = OriginModel::default();
    for at_ms in 1_000..=1_002 {
        model.observe(&OriginObservation::failure(
            full.clone(),
            at_ms,
            ErrorReason::Timeout,
        ));
    }
    let claim = model
        .claim(&full, 5_000, DecisionMode::Normal)
        .into_parts()
        .1
        .expect("recovery claim");

    model.observe(&OriginObservation::success(full.clone(), 5_100));

    assert_eq!(model.circuit_admission(&full, 5_101), Admission::Blocked);
    let sparse = OriginObservation::success(query(RequestMethod::RangeGet, 65_536), 5_102);
    assert!(model.complete_claim(claim, AdmissionClaimTerminal::Observed(&sparse)));
    assert_eq!(
        model.circuit_admission(&full, 5_103),
        Admission::RecoveryTrial
    );
}

fn query(method: RequestMethod, bytes: u64) -> OriginQuery {
    OriginQuery::new(
        "https://recovered.example/video.mp4",
        OriginContext::new(method, bytes, MediaClass::WholeObject),
    )
}
