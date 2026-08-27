use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, MediaClass, OriginContext, OriginModel,
    OriginQuery, RequestMethod,
};

#[test]
fn started_exploration_without_observation_keeps_its_origin_lease() {
    let mut model = OriginModel::default();
    let query = OriginQuery::new(
        "https://cold.example/video.mp4",
        OriginContext::new(RequestMethod::RangeGet, 65_536, MediaClass::ProgressiveMp4),
    );
    let (_, claim) = model
        .claim(&query, 1_000, DecisionMode::Normal)
        .into_parts();
    let claim = claim.expect("exploration claim");

    assert!(model.complete_claim(claim, AdmissionClaimTerminal::StartedWithoutObservation));

    assert_eq!(
        model.claim(&query, 1_001, DecisionMode::Normal).admission(),
        Admission::Blocked
    );
}
