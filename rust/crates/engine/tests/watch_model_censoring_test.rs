use crate::watch_model::{
    WatchCensor, WatchContext, WatchKey, WatchModel, WatchSample, WatchSampleKind,
};

fn context(id: &str) -> WatchContext {
    WatchContext::new(WatchKey::digest(id), Some(10_000))
}

fn sample(id: &str, watched_ms: u64, kind: WatchSampleKind, at_ms: u64) -> WatchSample {
    WatchSample::new(context(id), watched_ms, kind, at_ms)
}

#[test]
fn abandonment_changes_hazard_while_censored_delivery_events_do_not_become_swipes() {
    let mut abandoned = WatchModel::default();
    let mut censored = WatchModel::default();
    for index in 0..8 {
        let at_ms = 1_000 + index * 100;
        abandoned.observe(&sample("clip", 1_000, WatchSampleKind::Abandoned, at_ms));
        censored.observe(&sample(
            "clip",
            1_000,
            WatchSampleKind::Censored(WatchCensor::TransportSubstitution),
            at_ms,
        ));
    }

    let abandoned_survival = abandoned.predict(&context("clip"), 2_000).survival(2_000);
    let censored_survival = censored.predict(&context("clip"), 2_000).survival(2_000);

    assert!(abandoned_survival < censored_survival);
    assert_eq!(abandoned.navigation_observations(), 0);
    assert_eq!(censored.navigation_observations(), 0);
}

#[test]
fn completions_are_right_censored_and_raise_late_watch_probability() {
    let mut complete = WatchModel::default();
    let mut early = WatchModel::default();
    for index in 0..8 {
        let at_ms = 10_000 + index * 100;
        complete.observe(&sample("clip", 10_000, WatchSampleKind::Completed, at_ms));
        early.observe(&sample("clip", 1_000, WatchSampleKind::Abandoned, at_ms));
    }

    assert!(
        complete.predict(&context("clip"), 20_000).survival(8_000)
            > early.predict(&context("clip"), 20_000).survival(8_000)
    );
}
