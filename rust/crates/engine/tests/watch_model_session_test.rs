use ghostr_engine::watch_model::{
    WatchContext, WatchKey, WatchModel, WatchNavigation, WatchSample, WatchSampleKind,
};

fn context(id: &str) -> WatchContext {
    WatchContext::new(WatchKey::digest(id), Some(20_000))
}

#[test]
fn consecutive_early_swipes_shift_the_session_then_completions_restore_continuation() {
    let mut model = WatchModel::default();
    let cold = model.predict(&context("next"), 0).p50_ms();
    for index in 0..3 {
        model.observe(WatchSample::new(
            context(&format!("early-{index}")),
            500,
            WatchSampleKind::Abandoned,
            1_000 + index,
        ));
        model.observe_navigation(WatchNavigation::Forward, 1_000 + index);
    }
    let changed_epoch = model.change_epoch();
    let early = model.predict(&context("next"), 2_000).p50_ms();

    for index in 0..5 {
        model.observe(WatchSample::new(
            context(&format!("complete-{index}")),
            20_000,
            WatchSampleKind::Completed,
            3_000 + index,
        ));
    }

    assert!(early < cold);
    assert!(changed_epoch > 0);
    assert!(model.predict(&context("next"), 4_000).p50_ms() > early);
}

#[test]
fn backward_navigation_updates_direction_probability_and_resets_fast_session_state() {
    let mut model = WatchModel::default();
    for index in 0..4 {
        model.observe_navigation(WatchNavigation::Forward, index);
    }
    let forward_before = model.navigation().forward_probability();
    model.observe_navigation(WatchNavigation::Backward, 10);

    assert!(model.navigation().forward_probability() < forward_before);
    assert!(model.navigation().backward_probability() > 0.0);
    assert_eq!(model.session_observations(), 0);
}
