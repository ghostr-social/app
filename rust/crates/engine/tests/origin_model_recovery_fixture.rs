use crate::origin_model::{
    Admission, AdmissionClaim, DecisionMode, ErrorReason, MediaClass, OriginAdmissionIntent,
    OriginContext, OriginModel, OriginObservation, OriginQuery, RequestMethod,
};

pub(super) const URL: &str = "https://recovered.example/video.mp4";

pub(super) fn query(method: RequestMethod, bytes: u64) -> OriginQuery {
    query_at(URL, method, bytes)
}

pub(super) fn query_at(url: &str, method: RequestMethod, bytes: u64) -> OriginQuery {
    OriginQuery::new(
        url,
        OriginContext::new(method, bytes, MediaClass::WholeObject),
    )
}

pub(super) fn open_circuit(query: &OriginQuery) -> OriginModel {
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

pub(super) fn claim(
    model: &mut OriginModel,
    query: &OriginQuery,
    at_ms: u64,
    expected: Admission,
) -> AdmissionClaim {
    let (admission, claim) = model
        .claim(
            query,
            at_ms,
            DecisionMode::Normal,
            OriginAdmissionIntent::Delivery,
        )
        .into_parts();
    assert_eq!(admission, expected);
    claim.expect("typed recovery claim")
}
