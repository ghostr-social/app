use crate::watch_model::{WatchContext, WatchKey, WatchModel, WatchProgress};
use core::time::Duration;

#[test]
fn remaining_watch_conditions_away_departures_that_already_did_not_happen() {
    let model = WatchModel::default();
    let contexts = [("current", 8_000), ("next", 12_000), ("third", 20_000)]
        .map(|(id, duration)| WatchContext::new(WatchKey::digest(id), Some(duration)));
    let cold = model.predict_window(&contexts, 1_000);
    let remaining = model.predict_remaining_window(
        &contexts,
        WatchProgress::normal(Duration::from_millis(7_000)),
        1_000,
    );
    let next = &remaining.candidates()[1];

    assert_eq!(
        next.play_start().p95_ms(),
        1_000,
        "remaining current duration"
    );
    assert_eq!(
        next.play_start().probability_by(998),
        0.0,
        "elapsed early-departure mass is excluded, not moved to zero"
    );
    assert_eq!(
        next.reach_probability(),
        cold.candidates()[1].reach_probability(),
        "watch conditioning does not invent navigation reach"
    );
    assert!(
        remaining.candidates()[2].play_start().p95_ms() <= 13_000,
        "the following full watch is convolved with the conditional remainder"
    );
}
