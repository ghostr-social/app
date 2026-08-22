use ghostr_engine::origin_model::{
    Admission, DecisionMode, ErrorReason, MediaClass, OriginContext, OriginModel,
    OriginObservation, OriginQuery, RequestMethod,
};

fn query() -> OriginQuery {
    OriginQuery::new(
        "https://small-provider.example/video.mp4",
        OriginContext::new(
            RequestMethod::RangeGet,
            1_000_000,
            MediaClass::ProgressiveMp4,
        ),
    )
}

#[test]
fn exploration_is_bounded_and_disabled_outside_normal_mode() {
    let mut model = OriginModel::default();
    let key = query();
    assert!(matches!(
        model.claim(&key, 1_000, DecisionMode::Normal),
        Admission::Exploration {
            maximum_bytes: 65_536
        }
    ));
    assert_eq!(
        model.claim(&key, 1_001, DecisionMode::Normal),
        Admission::Blocked
    );
    assert_eq!(
        model.claim(&key, 1_002, DecisionMode::Safety),
        Admission::Production
    );
}

#[test]
fn open_circuit_allows_one_sparse_backed_off_recovery_probe() {
    let mut model = OriginModel::default();
    let key = query();
    for at in 1..=3 {
        model.observe(OriginObservation::failure(
            key.clone(),
            at * 1_000,
            ErrorReason::Timeout,
        ));
    }
    assert_eq!(
        model.claim(&key, 3_100, DecisionMode::Normal),
        Admission::Blocked
    );
    assert!(matches!(
        model.claim(&key, 5_000, DecisionMode::Normal),
        Admission::RecoveryProbe {
            maximum_bytes: 65_536
        }
    ));
    assert_eq!(
        model.claim(&key, 5_001, DecisionMode::Normal),
        Admission::Blocked
    );
    model.observe(OriginObservation::success(key.clone(), 5_100));
    assert_eq!(
        model.claim(&key, 5_101, DecisionMode::Safety),
        Admission::Production
    );
}
