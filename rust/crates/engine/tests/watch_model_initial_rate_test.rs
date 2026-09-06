use crate::watch_model::{WatchContext, WatchKey, WatchModel, WatchProgress};
use core::num::NonZeroU16;
use core::time::Duration;

#[test]
fn accelerated_playback_at_the_start_has_a_shorter_wall_clock_horizon() {
    let contexts = [("current", 8_000), ("next", 12_000)]
        .map(|(id, duration)| WatchContext::new(WatchKey::digest(id), Some(duration)));
    let progress = WatchProgress::at_rate(
        Duration::ZERO,
        NonZeroU16::new(2_000).expect("positive rate"),
    );
    let prediction = WatchModel::default().predict_remaining_window(&contexts, progress, 1_000);
    assert_eq!(
        prediction.candidates()[1].play_start().p95_ms(),
        4_000,
        "zero elapsed does not erase the selected playback rate"
    );
}
