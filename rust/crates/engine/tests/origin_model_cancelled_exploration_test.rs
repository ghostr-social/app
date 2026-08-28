use crate::origin_model::{
    Admission, AdmissionClaimTerminal, DecisionMode, MediaClass, OriginAdmissionIntent,
    OriginContext, OriginModel, OriginQuery, RequestMethod,
};

#[test]
fn cancelled_unstarted_exploration_restores_its_origin_lease() {
    let mut model = OriginModel::default();
    let query = OriginQuery::new(
        "https://cold.example/video.mp4",
        OriginContext::new(RequestMethod::PrefixGet, 65_536, MediaClass::ProgressiveMp4),
    );
    let (admission, claim) = model
        .claim(
            &query,
            1_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        )
        .into_parts();
    assert_eq!(admission, Admission::Exploration);
    let claim = claim.expect("exploration claim");

    assert!(model.complete_claim(claim, AdmissionClaimTerminal::NotStarted));

    assert!(matches!(
        model
            .claim(
                &query,
                1_001,
                DecisionMode::Normal,
                OriginAdmissionIntent::OptionalExploration,
            )
            .admission(),
        Admission::Exploration
    ));
}
