use super::support::planned_with_model;
use crate::adaptive::PlannerCommand;
use crate::origin_model::{
    ErrorReason, MediaClass, OpenBodyObservation, OriginContext, OriginModel, OriginObservation,
    OriginQuery, RequestMethod,
};

const SOURCE: &str = "https://origin.example/media";
const AT_MS: u64 = 1_000;

#[test]
fn request_failures_do_not_penalize_an_already_open_body() {
    let mut origins = OriginModel::default();
    let continuation_before = promotion_success_bps(&origins);
    for offset in 0..8 {
        origins.observe(&OriginObservation::failure(
            query(),
            AT_MS + offset,
            ErrorReason::Dns,
        ));
    }
    assert_eq!(promotion_success_bps(&origins), continuation_before);
}

#[test]
fn open_body_failures_lower_only_continuation_success() {
    let mut origins = OriginModel::default();
    let ordinary_before = origins.estimate(&query(), AT_MS, normal()).success.selected;
    let continuation_before = promotion_success_bps(&origins);
    for offset in 0..8 {
        origins.observe_open_body(&OpenBodyObservation::failure(
            query(),
            AT_MS + offset,
            ErrorReason::Connection,
        ));
    }
    assert!(promotion_success_bps(&origins) < continuation_before);
    assert_eq!(
        origins.estimate(&query(), AT_MS, normal()).success.selected,
        ordinary_before
    );
}

fn promotion_success_bps(origins: &OriginModel) -> u16 {
    let (_, planned) = planned_with_model(Some(200_000), false, origins);
    planned
        .generated
        .actions
        .iter()
        .find(|action| matches!(action.command, PlannerCommand::Promote { .. }))
        .expect("promotion")
        .node
        .forecast
        .success_bps
}

fn query() -> OriginQuery {
    OriginQuery::new(
        SOURCE,
        OriginContext::new(RequestMethod::RangeGet, 200_000, MediaClass::ProgressiveMp4),
    )
}

fn normal() -> crate::origin_model::DecisionMode {
    crate::origin_model::DecisionMode::Normal
}
