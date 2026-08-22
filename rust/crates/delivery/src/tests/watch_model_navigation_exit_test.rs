use crate::delivery_events::FocusTransition;
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::{empty, focus};
use ghostr_engine::watch_model::WatchNavigation;

#[test]
fn leaving_the_focus_window_records_exit_and_resets_session_state() {
    let mut learner = WatchLearner::default();
    learner.focus(&focus(1, 0, FocusTransition::UserNavigation), 10);
    learner.focus(&empty(700), 20);

    assert_eq!(learner.last_navigation(), Some(WatchNavigation::Exit));
    assert_eq!(learner.model().session_observations(), 0);
    assert!(learner.model().navigation().exit_probability() > 0.0);
}
