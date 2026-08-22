use crate::delivery_events::FocusTransition;
use crate::qoe::WatchLearner;
use crate::tests::watch_model_fixture::focus;
use ghostr_engine::watch_model::WatchNavigation;

#[test]
fn forward_and_backward_focus_changes_update_distinct_session_truth() {
    let mut forward = WatchLearner::default();
    forward.focus(&focus(1, 0, FocusTransition::UserNavigation), 10);
    forward.focus(&focus(2, 500, FocusTransition::UserNavigation), 20);

    let mut backward = WatchLearner::default();
    backward.focus(&focus(1, 0, FocusTransition::UserNavigation), 10);
    backward.focus(&focus(0, 500, FocusTransition::UserNavigation), 20);

    assert_eq!(forward.last_navigation(), Some(WatchNavigation::Forward));
    assert_eq!(backward.last_navigation(), Some(WatchNavigation::Backward));
    assert!(forward.model().session_observations() > 0);
    assert_eq!(backward.model().session_observations(), 0);
    assert!(
        forward.model().navigation().forward_probability()
            > backward.model().navigation().forward_probability()
    );
}
