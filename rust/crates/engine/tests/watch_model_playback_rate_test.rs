use crate::watch_model::{WatchContext, WatchKey, WatchModel, WatchProgress};
use core::num::NonZeroU16;
use core::time::Duration;

#[test]
fn current_speed_retimes_the_conditional_remainder_before_future_full_watches() {
    let contexts = [("current", 8_000), ("next", 12_000), ("third", 20_000)]
        .map(|(id, duration)| WatchContext::new(WatchKey::digest(id), Some(duration)));
    let progress = WatchProgress::at_rate(
        Duration::from_secs(7),
        NonZeroU16::new(2_000).expect("positive rate"),
    );
    let prediction = WatchModel::default().predict_remaining_window(&contexts, progress, 1_000);
    let next = prediction.candidates()[1].play_start();
    assert_eq!(
        next.p95_ms(),
        500,
        "one media second takes half a wall second"
    );
    assert_eq!(
        next.probability_by(498),
        0.0,
        "past abandonment stays excluded"
    );
    assert_eq!(
        prediction.candidates()[2].play_start().p95_ms(),
        12_500,
        "future clips retain their default playback rate"
    );
}
