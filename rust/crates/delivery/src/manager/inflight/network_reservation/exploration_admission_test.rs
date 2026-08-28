use super::FinishedAction;
use crate::manager::inflight::CompletionStatus;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, OriginAdmissionIntent, OriginContext, OriginModel, OriginQuery,
    RequestMethod,
};

#[test]
fn finished_action_attributes_bytes_only_to_an_exploration_claim() {
    let query = OriginQuery::new(
        "https://media.example/video.mp4",
        OriginContext::new(RequestMethod::PrefixGet, 65_536, MediaClass::ProgressiveMp4),
    );
    let mut model = OriginModel::default();
    let (_, claim) = model
        .claim(
            &query,
            1_000,
            DecisionMode::Normal,
            OriginAdmissionIntent::OptionalExploration,
        )
        .into_parts();
    let explored = FinishedAction {
        status: CompletionStatus::Current,
        network_reservation: None,
        admission_claim: claim,
    };
    let delivered = FinishedAction {
        status: CompletionStatus::Current,
        network_reservation: None,
        admission_claim: None,
    };

    assert!(explored.exploration_admitted());
    assert!(!delivered.exploration_admitted());
}
