use crate::watch_model::{WatchContext, WatchKey, WatchModel, WatchSample, WatchSampleKind};

fn context(raw: &str) -> WatchContext {
    WatchContext::new(WatchKey::digest(raw), Some(15_000))
        .with_creator(WatchKey::digest("creator-secret"))
}

#[test]
fn bounded_aggregate_state_round_trips_without_raw_social_identifiers() {
    let mut model = WatchModel::default();
    for index in 0..600 {
        model.observe(&WatchSample::new(
            context(&format!("raw-post-{index}")),
            1_000 + index,
            WatchSampleKind::Abandoned,
            1_000 + index,
        ));
    }
    let json = model.state().to_json();
    let restored = WatchModel::from_state_json(&json).expect("state");

    assert!(!json.contains("raw-post"));
    assert!(!json.contains("creator-secret"));
    assert!(restored.persisted_group_count() <= WatchModel::MAX_PERSISTED_GROUPS);
    assert_eq!(restored.revision(), model.revision());
}

#[test]
fn calibration_labels_survive_restart_and_exclude_unknown_censored_tail_labels() {
    let mut exact = WatchModel::default();
    let mut censored = WatchModel::default();
    exact.observe(&WatchSample::new(
        context("exact"),
        1_000,
        WatchSampleKind::Abandoned,
        10,
    ));
    censored.observe(&WatchSample::new(
        context("censored"),
        1_000,
        WatchSampleKind::Completed,
        10,
    ));
    let restored =
        WatchModel::from_state_json(&exact.state().to_json()).expect("valid test fixture");

    assert_eq!(restored.calibration_labels(), exact.calibration_labels());
    assert!(exact.calibration_labels() > censored.calibration_labels());
    assert!(restored.calibration_error_bps() <= 10_000);
}
