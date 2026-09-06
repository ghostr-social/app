use crate::watch_model::{WatchContext, WatchKey, WatchModel, WatchProgress};
use core::time::Duration;

#[test]
fn elapsed_watch_beyond_the_modeled_support_does_not_manufacture_more_slack() {
    let model = WatchModel::default();
    let contexts = [
        WatchContext::new(WatchKey::digest("current"), Some(8_000)),
        WatchContext::new(WatchKey::digest("next"), Some(12_000)),
    ];
    let prediction = model.predict_remaining_window(
        &contexts,
        WatchProgress::normal(Duration::from_secs(10)),
        1_000,
    );
    let next = prediction.candidates()[1].play_start();

    assert_eq!(
        next.p95_ms(),
        0,
        "exhausted support uses immediate preparation"
    );
    assert_eq!(
        next.probability_by(0),
        1.0,
        "fallback is a normalized immediate distribution"
    );
}
