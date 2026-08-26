use crate::origin_model::{
    AdaptationState, DecisionMode, MediaClass, OriginContext, OriginModel, OriginObservation,
    OriginQuery, RequestMethod,
};

fn query() -> OriginQuery {
    OriginQuery::new(
        "https://changing.example/video.mp4",
        OriginContext::new(
            RequestMethod::RangeGet,
            256 * 1024,
            MediaClass::ProgressiveMp4,
        ),
    )
}

#[test]
fn abrupt_failure_switches_to_short_adaptation_and_recovers_long_term_weight() {
    let mut model = OriginModel::default();
    let key = query();
    for at in 1..=12 {
        model.observe(
            &OriginObservation::success(key.clone(), at * 1_000)
                .with_ttfb_ms(30)
                .with_throughput_bps(10_000_000),
        );
    }
    let healthy = model.estimate(&key, 13_000, DecisionMode::Safety);
    for at in 13..=15 {
        model.observe(&OriginObservation::failure(
            key.clone(),
            at * 1_000,
            crate::origin_model::ErrorReason::Timeout,
        ));
    }

    let changed = model.estimate(&key, 15_100, DecisionMode::Safety);
    assert_eq!(changed.adaptation, AdaptationState::Short);
    assert!(changed.success.selected < healthy.success.selected);
    let restored = model.estimate(&key, 3_700_000, DecisionMode::Safety);
    assert_eq!(restored.adaptation, AdaptationState::Long);
}

#[test]
fn safety_and_emergency_use_more_conservative_latency_and_throughput() {
    let mut model = OriginModel::default();
    let key = query();
    for (at, ttfb, throughput) in [(1, 20, 12_000_000), (2, 80, 4_000_000), (3, 200, 1_000_000)] {
        model.observe(
            &OriginObservation::success(key.clone(), at * 1_000)
                .with_ttfb_ms(ttfb)
                .with_throughput_bps(throughput),
        );
    }
    let normal = model.estimate(&key, 4_000, DecisionMode::Normal);
    let safety = model.estimate(&key, 4_000, DecisionMode::Safety);
    let emergency = model.estimate(&key, 4_000, DecisionMode::Emergency);
    assert!(normal.ttfb_ms.selected <= safety.ttfb_ms.selected);
    assert!(safety.ttfb_ms.selected <= emergency.ttfb_ms.selected);
    assert!(normal.throughput_bps.selected >= safety.throughput_bps.selected);
    assert!(safety.throughput_bps.selected >= emergency.throughput_bps.selected);
}

#[test]
fn sustained_latency_and_throughput_shift_activates_short_model() {
    let mut model = OriginModel::default();
    let key = query();
    for at in 1..=8 {
        model.observe(
            &OriginObservation::success(key.clone(), at * 1_000)
                .with_ttfb_ms(30)
                .with_throughput_bps(10_000_000),
        );
    }
    for at in 9..=11 {
        model.observe(
            &OriginObservation::success(key.clone(), at * 1_000)
                .with_ttfb_ms(800)
                .with_throughput_bps(100_000),
        );
    }

    let changed = model.estimate(&key, 11_100, DecisionMode::Safety);
    assert_eq!(changed.adaptation, AdaptationState::Short);
    assert!(changed.ttfb_ms.p50 > 30);
    assert!(changed.throughput_bps.p50 < 10_000_000);
}
