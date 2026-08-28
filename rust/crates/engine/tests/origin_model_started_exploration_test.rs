use crate::origin_model::{
    Admission, AdmissionBlockReason, AdmissionClaimTerminal, DecisionMode, MediaClass,
    OriginAdmissionIntent, OriginContext, OriginModel, OriginQuery, RequestMethod,
};

#[test]
fn started_exploration_consumes_the_origin_budget_but_not_safety_work() {
    let mut model = OriginModel::default();
    let query = OriginQuery::new(
        "https://cold.example/video.mp4",
        OriginContext::new(RequestMethod::PrefixGet, 65_536, MediaClass::ProgressiveMp4),
    );
    let (_, claim) = model
        .claim(
            &query,
            1_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        )
        .into_parts();
    let claim = claim.expect("exploration claim");

    assert!(model.complete_claim(claim, AdmissionClaimTerminal::StartedWithoutObservation));

    let blocked = model.claim(
        &query,
        1_001,
        DecisionMode::Normal,
        OriginAdmissionIntent::OptionalExploration,
    );
    assert_eq!(blocked.admission(), Admission::Blocked);
    assert_eq!(
        blocked.block_reason(),
        Some(AdmissionBlockReason::ExplorationBudgetExhausted)
    );
    assert_eq!(
        model
            .claim(
                &query,
                1_002,
                DecisionMode::Safety,
                OriginAdmissionIntent::OptionalExploration,
            )
            .admission(),
        Admission::Production
    );

    let delivery = OriginQuery::new(
        "https://cold.example/video.mp4",
        OriginContext::new(RequestMethod::RangeGet, 262_144, MediaClass::ProgressiveMp4),
    );
    assert_eq!(
        model
            .claim(
                &delivery,
                1_003,
                DecisionMode::Normal,
                OriginAdmissionIntent::Delivery,
            )
            .admission(),
        Admission::Production
    );
}

#[test]
fn exploration_preview_is_pure_and_exposes_the_exact_refill_boundary() {
    let mut model = OriginModel::default();
    let query = OriginQuery::new(
        "https://cold.example/video.mp4",
        OriginContext::new(RequestMethod::PrefixGet, 65_536, MediaClass::ProgressiveMp4),
    );
    let (_, claim) = model
        .claim(
            &query,
            1_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        )
        .into_parts();
    model.complete_claim(
        claim.expect("exploration claim"),
        AdmissionClaimTerminal::StartedWithoutObservation,
    );
    let before = serde_json::to_string(&model).expect("serializable model");

    assert_eq!(
        model.admission_block_reason(
            &query,
            60_999,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        ),
        Some(AdmissionBlockReason::ExplorationBudgetExhausted)
    );
    assert_eq!(
        model.admission_block_reason(
            &query,
            61_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        ),
        None
    );
    assert_eq!(
        serde_json::to_string(&model).expect("serializable model"),
        before
    );
}
