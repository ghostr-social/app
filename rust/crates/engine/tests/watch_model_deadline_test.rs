use crate::watch_model::{WatchContext, WatchKey, WatchModel};

fn context(id: &str, duration_ms: u64) -> WatchContext {
    WatchContext::new(WatchKey::digest(id), Some(duration_ms))
}

#[test]
fn play_start_particles_convolve_preceding_watch_distributions_with_explicit_reach() {
    let model = WatchModel::default();
    let contexts = [
        context("current", 8_000),
        context("next", 12_000),
        context("third", 20_000),
    ];
    let prediction = model.predict_window(&contexts, 1_000);

    assert_eq!(prediction.candidates()[0].play_start().p50_ms(), 0);
    assert!(prediction.candidates()[1].play_start().p50_ms() > 0);
    assert!(
        prediction.candidates()[2].play_start().p50_ms()
            >= prediction.candidates()[1].play_start().p50_ms()
    );
    assert!(
        prediction.candidates()[2].reach_probability()
            < prediction.candidates()[1].reach_probability()
    );
}

#[test]
fn later_unit_deadlines_add_offset_and_condition_reach_on_survival() {
    let model = WatchModel::default();
    let prediction = model.predict_window(&[context("current", 10_000)], 0);
    let candidate = &prediction.candidates()[0];
    let unit = candidate.unit_deadline(4_000);

    assert_eq!(
        unit.deadline().p50_ms(),
        candidate.play_start().p50_ms() + 4_000
    );
    assert!(unit.reach_probability() < candidate.reach_probability());
}
