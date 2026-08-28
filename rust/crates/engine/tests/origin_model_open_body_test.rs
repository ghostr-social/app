use crate::origin_model::{
    DecisionMode, ErrorReason, MediaClass, OpenBodyObservation, OriginContext, OriginModel,
    OriginObservation, OriginQuery, RequestMethod,
};

#[test]
fn a_body_failure_never_increases_selected_success_in_any_control_mode() {
    let query = OriginQuery::new(
        "https://media.example/video.mp4",
        OriginContext::new(RequestMethod::RangeGet, 200_000, MediaClass::ProgressiveMp4),
    );
    let mut model = OriginModel::default();
    let before = estimates(&model, &query, 1_000);
    model.observe_open_body(&OpenBodyObservation::failure(
        query.clone(),
        1_001,
        ErrorReason::Connection,
    ));
    let after = estimates(&model, &query, 1_001);
    for (before, after) in before.into_iter().zip(after) {
        assert!(after < before, "failure raised {before:.4} to {after:.4}");
    }
}

#[test]
fn cancelled_body_keeps_observed_range_semantics_without_labeling_request_success() {
    let query = OriginQuery::new(
        "https://range-blind.example/video.mp4",
        OriginContext::new(RequestMethod::RangeGet, 64_000, MediaClass::ProgressiveMp4),
    );
    let mut model = OriginModel::default();
    let before = model.estimate(&query, 1_000, DecisionMode::Normal);
    model.observe(&OriginObservation::cancelled(query.clone(), 1_001).with_range_compliance(false));
    let after = model.estimate(&query, 1_001, DecisionMode::Normal);

    assert_eq!(after.success.mean, before.success.mean);
    assert!(
        after.range_compliance.expect("range model").mean
            < before.range_compliance.expect("range model").mean
    );
}

fn estimates(model: &OriginModel, query: &OriginQuery, at_ms: u64) -> Vec<f64> {
    modes()
        .map(|mode| {
            model
                .estimate_open_body(query, at_ms, mode)
                .success
                .selected
        })
        .collect()
}

fn modes() -> impl Iterator<Item = DecisionMode> {
    [
        DecisionMode::Normal,
        DecisionMode::Safety,
        DecisionMode::Emergency,
    ]
    .into_iter()
}
