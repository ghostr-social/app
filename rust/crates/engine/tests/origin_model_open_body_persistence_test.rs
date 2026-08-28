use crate::host_stats::HostStats;
use crate::origin_model::{
    DecisionMode, MediaClass, OpenBodyObservation, OriginContext, OriginObservation, OriginQuery,
    RequestMethod,
};

#[test]
fn nonempty_open_body_model_round_trips() {
    let query = body_query();
    let mut stats = HostStats::new();
    stats
        .origin_model_mut()
        .observe_open_body(&OpenBodyObservation::success(query.clone(), 1_000));
    let encoded = stats.to_json();
    assert!(encoded.contains("open_body_origins"));

    let loaded = HostStats::from_json(&encoded).expect("valid open-body model");
    let estimate = loaded
        .origin_model()
        .estimate_open_body(&query, 1_001, DecisionMode::Normal);
    assert!(estimate.effective_samples > 0.0);
}

#[test]
fn empty_open_body_maps_preserve_the_legacy_json_shape() {
    let query = OriginQuery::new(
        "https://legacy.example/video.mp4",
        OriginContext::new(RequestMethod::FullGet, 900_000, MediaClass::WholeObject),
    );
    let mut stats = HostStats::new();
    stats
        .origin_model_mut()
        .observe(&OriginObservation::success(query, 1_000));
    let encoded = stats.to_json();
    assert!(!encoded.contains("open_body_"));

    let loaded = HostStats::from_json(&encoded).expect("legacy model loads");
    assert_eq!(loaded.to_json(), encoded);
}

fn body_query() -> OriginQuery {
    OriginQuery::new(
        "https://persistent.example/video.mp4",
        OriginContext::new(RequestMethod::RangeGet, 200_000, MediaClass::ProgressiveMp4),
    )
}
