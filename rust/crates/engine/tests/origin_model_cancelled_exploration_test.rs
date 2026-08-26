use crate::origin_model::{
    Admission, DecisionMode, MediaClass, OriginContext, OriginModel, OriginQuery, RequestMethod,
};

#[test]
fn cancelled_unstarted_exploration_restores_its_origin_lease() {
    let mut model = OriginModel::default();
    let query = OriginQuery::new(
        "https://cold.example/video.mp4",
        OriginContext::new(
            RequestMethod::RangeGet,
            64 * 1024,
            MediaClass::ProgressiveMp4,
        ),
    );
    let Admission::Exploration { claim, .. } = model.claim(&query, 1_000, DecisionMode::Normal)
    else {
        panic!("cold origin must receive exploration admission");
    };

    model.release_exploration(&claim);

    assert!(matches!(
        model.claim(&query, 1_001, DecisionMode::Normal),
        Admission::Exploration { .. }
    ));
}
