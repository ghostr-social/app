use crate::delivery_events::FocusTransition;
use crate::manager::state::DeliveryState;
use crate::tests::focus_navigation_history_test::focus;
use ghostr_engine::{DataUsageLevel, EngineParams};

#[test]
fn a_transport_rescue_does_not_train_the_user_swipe_model() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    assert!(state.apply_focus(focus(0), 1_000));
    let mut rescue = focus(2);
    rescue.transition = FocusTransition::TransportRescue;

    assert!(state.apply_focus(rescue, 2_000));

    let navigation = state.navigation(2_000);
    assert_eq!(navigation.forward_swipes_per_minute, 0);
    assert_eq!(navigation.backward_swipes_per_minute, 0);
}
